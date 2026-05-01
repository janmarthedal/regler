# Changelog

Per-version log of program changes. Versions match `Cargo.toml`.

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
