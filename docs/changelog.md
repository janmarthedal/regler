# Changelog

Per-version log of program changes. Versions match `Cargo.toml`.

## 0.7.0

Milestone 8: sets as first-class objects, set-builder definitions, and membership discharge.

- **Opaque set declarations.** `let S : Set` declares `S` as an opaque set. No rewrite rule or kernel structure is generated; the declaration is accepted and printed.
- **Predicate set definitions.** `let Name : Set = {x ∈ S | P}` defines a set by a subset comprehension. The kernel stores the bound variable `x`, domain `S`, and predicate `P`. Sets defined this way participate in the membership-discharge mechanism.
- **Function signature declarations.** `let f : S → T` declares `f` as a function from `S` to `T`. Accepted and printed; no rewriting machinery is installed for the signature itself.
- **Subset facts.** `fact S ⊆ T` is parsed and accepted as a `SubsetFact` without installing a rewrite rule. Subset reasoning is stored for future use.
- **Membership discharge in conditions.** `condition_holds` now handles `e ∈ S` when `S` is a predicate-defined set: it substitutes `e` for the bound variable in `S`'s predicate and checks the result. Numeric comparisons `>`, `<`, `≥`, `≤`, `=` on rational literals and boolean connectives `∧`, `∨` are now all supported in conditions.
- **Binder-generated conditions.** When `fact ∀ vars ∈ S. body` is installed and `S` is a predicate-defined set, the kernel automatically generates membership side conditions `v ∈ S` for each bound variable `v`. These are merged with any explicit `if` clause. Facts over opaque sets (ℕ, ℤ, ℚ, ℝ, …) are unaffected — their domain annotation remains informational.
- **`apply` checks conditions.** `apply name to expr` now verifies the named fact's side condition (if any) under the match substitution before rewriting. If the condition is not decidably satisfied at a position, that position is skipped and the search continues into subterms. `apply_eq_conditional` is added to `kernel::rewrite` for this purpose.
- **Top-down rewriting in `simplify`.** The simplification loop now tries user rewrite rules at the current node *before* recursing into subterms (in addition to the existing post-bottom-up pass). This ensures that rules with compound subterm patterns (e.g. `log(a·b)`) are tried before inner arithmetic reduces the argument.
- **Function application syntax.** `f(a, b, …)` is now parsed as `Expr::App` and lowered to `Term::App`. `to_surface` converts `Term::App` nodes with non-operator heads back to `Expr::App` for round-trip printing.
- **Set-builder syntax.** `{x ∈ S | P}` is parsed as `Expr::SetBuilder` and printed faithfully. It cannot be lowered to a kernel term (it is only valid as a definition RHS); other uses produce a lower error.
- **New surface operators:** `→` (function type, prec 45, right-assoc), `⊆` (subset, prec 40), `∈` (membership, prec 40), `<`, `>`, `≤`, `≥` (comparisons, prec 40). All non-associative at the comparison level.
- **`Command::Let` extended.** The let command now accepts an optional type annotation and an optional RHS: `let name [: type] [= rhs]`. All three forms (declaration-only, annotation+definition, bare definition) are parsed, printed, and round-trip correctly.
- New runnable example `examples/log.rgl` exercising the full milestone: `ℝ : Set`, `fact ℚ ⊆ ℝ`, `Pos = {x ∈ ℝ | x > 0}`, `log : Pos → ℝ`, the log-product fact, `apply log_product to log(2·3)` → `log(2) + log(3)`, and `simplify log(2) + log(3)` → `log(6)`.

## 0.6.0

Milestone 7: side conditions on facts, named facts, and `apply`/`apply ←`.

- **Named facts.** `fact name : proposition` gives a fact an optional name. Named facts are stored in `Theory::named` (a `HashMap<Symbol, NamedFact>`) with the as-written `(lhs, rhs)` direction preserved, independently of how KBO orients the auto-rule. Anonymous facts work exactly as before.
- **Side conditions.** `fact prop if cond` attaches a condition to a fact. Conditions are arbitrary expressions; `≠` (not-equal) is the first condition operator supported. A conditional auto-rule only fires during `simplify` when the condition is verifiably true under the match substitution — both sides of `≠` must evaluate to distinct numeric literals. When the condition cannot be decided (e.g. a symbolic operand), the rule is conservatively skipped.
- **`apply name to expr` command.** Applies a named fact in its as-written direction (LHS as pattern, RHS as replacement) to the first matching subterm of `expr`, searching top-down leftmost-outermost. Prints the rewritten term. Prints an error if no subterm matches.
- **`apply ← name to expr` command.** Same as `apply` but swaps lhs and rhs, letting the user run any fact in reverse regardless of how KBO would orient it. This is the primary mechanism for manually invoking equalities that are KBO-incomparable (factor/expand pairs, etc.).
- **AC recognition is suppressed for conditional facts.** A fact whose shape matches commutativity or associativity is only promoted to an AC mark when it carries no `if` clause, matching the design note in `CLAUDE.md`.
- **New `Op::Ne` (`≠`) surface operator** at the same precedence level as `=` (non-associative). Recognized by the lexer as the Unicode character `≠`; printed and parsed identically; lowered to `App("≠", [l, r])` in the kernel.
- **New tokens and keywords:** `≠` (NotEquals), `:` (Colon), `←` (LeftArrow), `apply`, `to`, `if`.
- **`Rule` gains `condition: Option<Term>`.** Existing code that constructs `Rule` values via `orient` is unaffected — `orient` always sets `condition: None`; the condition is attached separately during `install_fact`.
- New runnable example `examples/apply.rgl` demonstrating named facts, forward and reverse `apply`, and conditional rules firing or being blocked by the `≠` gate.
- Integration tests in `tests/apply.rs` covering: `apply_eq` top-level and subterm matching, no-match returning `None`, reverse application, named and anonymous fact parsing/printing round-trips, conditional fact round-trips, `apply`/`apply ←` command parsing/printing, condition gate (fires for non-zero literal, blocked for zero and for symbolic), named fact storage in theory, and commutativity-shaped named fact not installing a rule.

## 0.5.0

Milestone 6: widen numeric tower to ℤ and ℚ.

- `Term` gains two new variants: `Int(BigInt)` for negative integers and `Rat(BigRational)` for non-integer rationals. Variant order is `App < Var < Nat < Int < Rat`, preserving the "literals sort last" invariant used by AC normalization.
- Arithmetic is now closed over ℕ ∪ ℤ ∪ ℚ with implicit promotion: any binary operation on numeric literals uses `BigRational` internally and converts the result back to the most specific type (`Nat` if non-negative integer, `Int` if negative integer, `Rat` otherwise). This applies both in `evaluate` and in `simplify`'s literal-folding path.
- New operators `-` (subtraction) and `/` (division) at the surface level. `-` has the same precedence as `+` (level 2); `/` has the same precedence as `·` (level 3); both are left-associative. Division by zero is reported as an error in `evaluate`.
- Unary minus on integer literals: `-3` in surface syntax folds into `Expr::Int(-3)` at parse time, so negative integer results round-trip correctly through print → parse.
- Negative integer literals (`Expr::Int` with negative value) lower to `Term::Int`; non-negative integers continue to lower to `Term::Nat`.
- KBO extended: `Int` and `Rat` have weight 1 (like `Nat`); numeric literals of any mix of types are compared by their rational value; the builtin-precedence table adds `-` at level 1 (with `+`) and `/` at level 2 (with `·`).
- `pmatch`, `subst`, `theory::is_closed`, `simplify`, `fold_literals`, and kernel-to-surface printing all extended for `Int` and `Rat`.
- `fold_literals` in AC normalization now accumulates via `BigRational`, so AC `+`/`·` operators fold mixed-type literal operands correctly.
- Added `num-rational` dependency.
- Integration tests in `tests/numeric_tower.rs` covering: subtraction with Nat/zero/negative results, negative literal parsing, Int-to-Nat promotion on addition, exact and fractional division, negative rationals, mixed rational arithmetic, and unchanged ℕ behaviour.

## 0.4.0

Milestone 5: AC recognition and identity-element marking.

- New `kernel::theory::Theory` value gathers facts into the kernel's working theory: rewrite rules earned by KBO orientation, AC marks, and identity-element marks. `Theory::install_fact` recognises shape-specific facts (commutativity, associativity, left/right identity) before falling back to KBO orientation, and reports every effect a fact produced so the REPL can print notes (e.g. ``recognised commutativity for `+` ``, ``` `+` promoted to AC ```).
- AC marking is earned dynamically: when both a commutativity-shape fact (`f(a,b) = f(b,a)`) and an associativity-shape fact (`f(f(a,b),c) = f(a,f(b,c))`, in either direction) have been seen for the same `f`, `f` is promoted to AC. Neither fact installs a rewrite rule on its own — they only set the marks.
- Identity-element marking is earned similarly. `f(x, e) = x` registers `e` as a right identity for `f`; `f(e, x) = x` registers a left identity. When `f` is AC the two coincide, so a single fact covers both sides.
- `simplify` now takes a `&Theory` and applies, in order: literal arithmetic on ℕ, AC normalisation for AC heads (flatten nested same-head applications, drop identity operands, fold contiguous numeric operands of `+`/`·`, sort by the canonical term order, collapse 0/1-operand results), binary identity-operand absorption for non-AC heads, and KBO-oriented rewrite rules.
- `Term` now derives `Ord` for canonical sorting. Variant order is `App < Var < Nat`, which puts numeric constants last in printed output (`a + 5`, not `5 + a`).
- The kernel-to-surface printer now refolds n-ary applications with binary-op heads into left-associative binary chains so AC-flattened terms still round-trip to readable surface syntax.
- New runnable example `examples/ac.rgl` exercising commutativity, associativity, identity, AC normalisation, and literal folding for both `+` and `·` against the actual REPL.
- Integration tests covering: commutativity-only and associativity-only do not promote, AC promotion canonicalises operand order, AC literal folding, identity-operand absorption inside AC operators, AC collapse to a lone operand or to the identity element, AC for `·` independent of `+`, non-AC left-identity dropping, and AC unifying the two identity sides from a single right-identity fact.

## 0.3.0

Milestone 4: auto-oriented rewriting.

- KBO (Knuth-Bendix order) with default weight 1 for every variable, every numeric literal, and every function symbol. Comparison enforces the standard variable-count constraint. Precedence on App heads orders the four built-ins as `= < + < · < ^` (mirroring surface precedence); unknown heads fall back to byte-wise string order and rank above the built-ins.
- Pattern matching against kernel terms: every `Var` in a pattern is a pattern variable, repeated occurrences must bind to syntactically equal terms, and matching is functional (no rollback hazards on partial failure).
- New `Rule { lhs, rhs }` and `orient` API: an equality is auto-oriented toward its KBO-smaller side. KBO-incomparable equalities (e.g. commutativity `a + b = b + a`) and trivial equalities (`x = x`) report a note and install no rule. The KBO orientation guarantees every variable in `rhs` also appears in `lhs`, so equalities like `x = y` are correctly rejected.
- New `simplify` REPL command: lower → substitute `let` bindings → repeatedly apply every installed rule plus closed literal arithmetic on ℕ, until fixed point. Both rewriting and arithmetic strictly decrease the term in KBO, so the loop terminates.
- `fact` now lowers the surface expression and, if it is an equality at the root, attempts to install it as a rule. Non-equality facts are stored unchanged. `Var`s inside a fact are pattern variables and are NOT resolved against `let` bindings — `fact x + 0 = x` orients with `x` as a free pattern variable.
- Integration tests covering KBO direction, incomparability for commutativity, orientation regardless of written direction, rejection of unbound-rhs equalities, rejection of trivial equalities, rewriting under context, rewrite/arithmetic interleaving, let-binding resolution before rewriting, and round-trip parsing of the new `simplify` command.

## 0.2.0

Milestone 3: kernel term representation and `evaluate`.

- New `kernel` module with a uniform-prefix `Term` ADT (`Nat(BigUint)`, `Var(Symbol)`, `App(Symbol, Vec<Term>)`). Symbols are `Rc<str>`; an interner is deferred until rewriting hot paths exist.
- Lowering from surface AST to kernel terms. Integer literals lower to `Nat`; binary operators lower to `App` with the operator symbol as head. Negative literals are rejected (no `Int` until ℤ arrives in milestone 6).
- Capture-free substitution `subst(&Term, &HashMap<Symbol, Term>)` over the kernel. No binders yet, so substitution is a plain tree walk.
- Bottom-up `evaluate` performing literal arithmetic on ℕ for `+`, `·`, and `^`. Closed integer subterms reduce; symbolic subterms are preserved (e.g. `x + 2 · 3` becomes `x + 6`). Exponents must fit in `u32`.
- Kernel-to-surface printing reuses the existing pretty-printer by lowering kernel terms back into `ast::Expr`.
- New `evaluate <expr>` REPL command. `let` bindings are substituted into the expression before evaluation, so `let x = 7` followed by `evaluate x · x + 1` prints `50`.
- Added `num-traits` dependency for `ToPrimitive` on `BigUint`.
- Integration tests for arithmetic, big-integer cases, partial symbolic reduction, evaluator identity on purely symbolic inputs, and substitution-resolved bindings.

## 0.1.0

Milestone 2: round-trip REPL on a minimal surface.

- Cargo crate scaffolded with `num-bigint` for arbitrary-precision integer literals.
- Lexer for identifiers, integer literals, `+`, `·`, `^`, `=`, `(`, `)`, and the keywords `let`, `fact`, `print`.
- AST with a single `BinOp` variant; `=` is a regular binary operator at the lowest precedence. Precedence order: `= < + < · < ^`. `^` is right-associative; `+` and `·` are left-associative; `=` is treated as non-associative by the printer.
- Pratt-style precedence-climbing parser. `fact` accepts any expression (kernel will validate fact shape later).
- Pretty-printer that parenthesizes only when needed to preserve `parse(print(t)) == t`.
- Line-based REPL accepting `let name = expr`, `fact <expr>`, and `print <expr>`. `print` resolves a bare identifier through `let` bindings (one level); everything else prints verbatim. Facts are stored but not yet used.
- Round-trip integration tests covering atoms, flat binops, precedence mixes, associativity, equality at top level, and commands.
