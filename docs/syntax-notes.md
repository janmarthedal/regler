# Syntax notes

Working notes on concrete syntax. Decisions here are tentative — recorded so the design conversation can resume later without rederiving everything.

## Bindings (`let`)

A single keyword `let` introduces every named thing — values, functions, and sets. Forms:

- `let name : Sort`                  — declaration (opaque; characterized later by facts).
- `let name : Sort = Expr`           — definition with explicit sort.
- `let name = Expr`                  — definition with sort inferred from `Expr`.
- `let name : Sort = Expr in body`   — local definition (expression-level binder).
- `let name = Expr in body`          — local definition with inferred sort.

The annotation is **required** for declarations (no RHS to infer from) and **optional** for definitions. The annotation is the set the value belongs to (ℕ, ℝ, ℝ → ℝ, …) or the universe `Set` for set-valued things.

### Type inference rule

When the annotation is omitted, the kernel infers the *smallest containing set*: each subexpression is given the most specific type from its constituents and operator signatures, walking up subset chains only when an operator's signature demands it.

| Expression       | Inferred type |
|------------------|---------------|
| `42`             | `ℕ`           |
| `1/2`            | `ℚ`           |
| `π`              | `ℝ`           |
| `π + i`          | `ℂ`           |
| `(1, 2)`         | `ℕ × ℕ`       |
| `(x : ℝ) ↦ 2·x`  | `ℝ → ℝ`       |

Writing an explicit annotation that is *wider* than the inferred type is always allowed — the kernel verifies membership via subset coercion. Writing one that is *narrower* (e.g., `let small : Pos = 1/2`) creates a proof obligation; that mechanism is deferred.

There is no function- or set-definition sugar. A function (or parameterized set) defined by an equation is always written as a declaration plus a `fact`. Sugar may be reintroduced later if it proves consistently useful.

## Statement separation

Statements are separated by newlines, with **indentation as continuation**:

- A non-empty, non-comment line starts a new statement *unless* its indent is strictly greater than the indent of the current statement's first line. In that case, it is a continuation.
- A line whose indent is less than or equal to the current statement's first-line indent ends the current statement and (if non-empty) starts the next one.
- Blank lines and comment-only lines do not affect statement boundaries.

```
let UnitInterval : Set =
    {x ∈ ℝ | 0 ≤ x ∧ x ≤ 1}

fact ∀ x, y ∈ ℝ.
    log(x · y) = log(x) + log(y)
    if x > 0 ∧ y > 0

fact ℕ ⊆ ℤ
```

There is no explicit terminator (no `;`). The `;` may be added later as an opt-in override (`let a : ℕ = 1; let b : ℕ = 2` on one line); not part of the core syntax for now.

Tabs and spaces are both whitespace, but mixing them in indentation is undefined behaviour at this stage — pick one and stick with it. (A formal rule may be added later.)

## Comments

Line comments start with `#` and run to end of line. There are no block comments.

```
# this is a comment
let π : ℝ   # trailing comment
```

## Identifiers

An identifier is a non-empty sequence of characters where:

- **First character:** any Unicode letter (general category `L*`) or `_`. Covers ASCII a–z A–Z, Greek (α–ω, Α–Ω), blackboard bold (ℕ ℤ ℚ ℝ ℂ), calligraphic (𝒮 𝒫), Fraktur, Hebrew (ℵ), etc.
- **Subsequent characters:** any Unicode letter, decimal digit (0–9), Unicode subscript digit (₀–₉) or letter, Unicode superscript digit (⁰–⁹) or letter (`⁺`, `⁻` included), `_`, or `'`.

Identifiers are case-sensitive: `f ≠ F`.

Examples that are valid: `x`, `f'`, `f''`, `x₁`, `factorial`, `ℝⁿ`, `ℚ⁺`, `α₁'`, `_tmp`, `Σ_n`.
Examples that are not: `2x` (digit leading), `x-y` (hyphen), `x.y` (dot), `f+g` (operator), `x y` (space).

### Notable consequences

- **Subscripts are part of the identifier**, not syntax. `x₁` is a single name; `x_i` makes `i` a literal subscript, not a variable. Indexed-by-variable use needs explicit application: `x(i)`.
- **Superscripts are part of the identifier too.** `x²` is an identifier, not `x ^ 2`. Write powers with `^` (`x^2`). A future lex-time rewrite for `x²` → `x^2` is possible but not part of the core.
- **Operator characters never appear in identifiers** — no `+`, `-`, `·`, `/`, `^`, `*`, `=`, `<`, `>`, `&`, `|`, hyphens, or whitespace.

### Reserved words

Identifiers that cannot be redefined: `let`, `fact`, `in`, `if`, `then`, `else`, `Set`.
Operator-like reserved tokens (not identifiers but worth listing): `∀`, `∃`, `λ`, `↦`.
The list will grow as the language fills in.

Standard-prelude names like `ℕ`, `ℤ`, `ℚ`, `ℝ`, `ℂ` are not reserved — they are identifiers defined in a library and could in principle be shadowed.

## Numeric literals

- **Integer literals**: a non-empty sequence of decimal digits (`0`, `1`, `42`, `1234567890`). Arbitrary precision.
- **No sign in literals**: `-3` is always the expression `-(3)`.
- **No alternative bases**: no `0x…`, `0b…`, `0o…`. Out of scope for a CAS.
- **No decimal or floating-point literals**: rejected by the lexer with a pointer to the rational form.
- **No separate rational literal form**: `p/q` is the expression `p / q`, with `/` between integer values producing a rational. The kernel canonicalizes to `gcd(p, q) = 1`, `q > 0`. Pattern matching that wants "any rational literal" inspects the structural application, not a single atom.
- **Digit grouping with `_` deferred** — not needed yet, and interacts with the underscore in identifiers; revisit if large constants become hard to read.

## Expression grammar

Operators are grouped into three layers — terms (numeric and set-valued), atomic propositions (relations), and compound propositions (logic) — with binders on top.

### Precedence (tightest first)

| Level | Operators / forms | Assoc. |
|---|---|---|
| 1 | atoms: identifiers, literals, `(e)`, `{…}`, tuple `(e₁, e₂, …)` | — |
| 2 | function application `f(x, y)` | left |
| 3 | unary `-x`, logical `¬P` | prefix |
| 4 | power `x ^ y` | right |
| 5 | multiplicative `·`, `/` | left |
| 6 | additive `+`, binary `-` | left |
| 7 | set difference `\` | left |
| 8 | set intersection `∩` | left |
| 9 | set union `∪` | left |
| 10 | Cartesian product `×` | right |
| 11 | function arrow `→` | right |
| 12 | comparisons `=`, `≠`, `<`, `≤`, `>`, `≥`, `∈`, `∉`, `⊆`, `⊇` | non-associative |
| 13 | conjunction `∧` | left |
| 14 | disjunction `∨` | left |
| 15 | implication `⇒` | right |
| 16 | biconditional `⇔` (if used) | non-associative |
| 17 | binders: `∀ x ∈ S. P`, `∃ x ∈ S. P`, `λ x : T. body`, `let x : T = e in body`, `if P then a else b` | extends rightward |

### Decisions implied by the table

- **Power is right-associative.** `a ^ b ^ c = a ^ (b ^ c)`.
- **`×` and `→` are right-associative**, so `A × B × C` = `A × (B × C)` and `A → B → C` = `A → (B → C)`. Combined with their precedences, `(A × B) → C` needs no parentheses.
- **Comparisons are non-associative.** `a < b < c` is a *parse error*; write `a < b ∧ b < c`. Avoids the `(a < b) < c` pitfall. Chained-comparison sugar may be added later; not core.
- **`=` is just a comparison** at level 12, used uniformly in facts and expressions. No separate equality form.
- **Binders extend rightward as far as possible.** `∀ x ∈ ℝ. P ∧ Q` parses as `∀ x ∈ ℝ. (P ∧ Q)`. Parentheses limit scope.
- **Unary `-` and binary `-` share the symbol.** `-3` is always the expression `-(3)`; there are no negative integer literals. The kernel canonicalizes internally.
- **No implicit multiplication.** `2x` is not `2·x`; the `·` is required.
- **No assignment**, so `=` is unambiguously equality.

### Things deferred

- **Superscript powers** (`x²`) — depends on identifier rules; defer.
- **Inline `if then else`** is listed at level 17 but its necessity is open; conditional behavior can be encoded via separate facts with `if` side conditions for now.

## Sets

### Decisions so far

- Sets are values declared with `let`; their sort is the universe `Set`. They can be named, passed as arguments, returned from functions.
- **First-class but bounded.** A fixed vocabulary of operations (`∪`, `∩`, `\`, `×`, `→`, set-builder) is provided. `Set` itself is a universe, not a member of any set — you cannot write `Set : Set`.
- **No declaration-time constraint sugar.** `let ℝ : Set ⊇ ℚ` is *not* allowed. The verbose form `let ℝ : Set; fact ℚ ⊆ ℝ` is required. This keeps declarations and facts cleanly separated.
- **Six conceptual forms** of set declaration/definition (see below).

### The forms

```
# 1. Bare opaque declaration
let ℝ : Set

# 2. Declaration plus separate fact statements
let ℝ : Set
fact ℚ ⊆ ℝ

let ℂ : Set
fact ℝ ⊆ ℂ

# 3. Definition by enumeration (extensional)
let Digits : Set = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9}
let Bit    : Set = {0, 1}

# 4. Definition by predicate (subset comprehension)
let Pos     : Set = {x ∈ ℝ | x > 0}
let Nonzero : Set = {x ∈ ℝ | x ≠ 0}

# 5. Definition by image
let Squares : Set = {n² | n ∈ ℕ}
let Evens   : Set = {2·k | k ∈ ℤ}

# 6. Image with filter (combined)
let EvenSquares : Set = {n² | n ∈ ℕ, n mod 2 = 0}

# 7. Definition by set algebra
let NonzeroReals : Set = ℝ \ {0}
let ℚ⁺           : Set = ℚ ∩ Pos
let RealPairs    : Set = ℝ × ℝ
let RealEndo     : Set = ℝ → ℝ

# 8. Parameterized set (a function returning Set; declaration + fact)
let Interval : ℝ × ℝ → Set
fact ∀ a, b ∈ ℝ. Interval(a, b) = {x ∈ ℝ | a ≤ x ∧ x ≤ b}

let Multiples : ℤ → Set
fact ∀ n ∈ ℤ. Multiples(n) = {n·k | k ∈ ℤ}

# 9. Parameterized over a set (Set as a sort/universe)
let Pairs : Set × Set → Set
fact ∀ S, T ∈ Set. Pairs(S, T) = S × T

let Endo : Set → Set
fact ∀ S ∈ Set. Endo(S) = S → S
```

("Six conceptual forms" refers to the categories: opaque declaration, extensional, predicate-subset, image, set-algebra, parameterized.)

### Usage examples

```
# Membership claims
1/2 ∈ ℚ
π   ∈ ℝ \ ℚ
0   ∈ Bit

# Subset claims (in facts or theorems)
fact ℕ ⊆ ℤ
fact ℤ ⊆ ℚ

# Function signatures (sets as domain/codomain)
let f     : ℝ → ℝ
let g     : ℝ × ℝ → ℝ
let sin   : ℝ → Interval(-1, 1)
let recip : Nonzero → ℝ

# Inline (anonymous) sets inside a signature
let abs_inv : {x ∈ ℝ | x ≠ 0} → Pos

# Variable bindings in facts
fact ∀ x ∈ ℝ. x + 0 = x
fact ∀ x, y ∈ ℝ. x + y = y + x

# Side conditions
fact ∀ a, b ∈ ℝ. log(a·b) = log(a) + log(b)   if a ∈ Pos ∧ b ∈ Pos

# Parameterized sets used like any function call
let UnitInterval : Set = Interval(0, 1)
let clamp        : ℝ → Interval(0, 1)
let m            : Endo(ℝ)

# Set algebra inline
let to_rat      : ℚ ∩ Pos → ℚ
let union_check : ℕ ∪ {-1, -2} → ℤ

# Set-builder used directly without naming
let sum_over : {n ∈ ℕ | n ≤ 10} → ℕ
```

### Open questions

- **The declaration-then-fact pattern.** Verbose for long subset chains (ℕ ⊆ ℤ ⊆ ℚ ⊆ ℝ ⊆ ℂ requires 4 separate facts) and for parameterized sets. Acceptable for now; revisit if it becomes painful in real examples.
- **ASCII fallbacks.** Whether `in`, `subset`, `forall`, etc. are accepted alongside the Unicode forms — deferred.
- **Sort of `Set`.** Treated as a universe: `S : Set` is a sort annotation in `let`, and `∀ S ∈ Set. P` is binding-shorthand under a quantifier, but `S ∈ Set` is *not* a writable proposition. Whether the language ever needs a higher universe is deferred — not needed for current goals.

## Facts

### Decisions so far

- Keyword: `fact`. Used to assert any statement the system should treat as given — equalities, subset claims, membership claims, and the defining equations of declared functions and parameterized sets.
- **One keyword for all asserted statements.** The syntax does not distinguish "axioms" (taken as fundamental) from "definitions" (introducing meaning); both are facts the kernel is told. A future `theorem` keyword may be added for proved statements.
- **Variables are bound by an explicit `∀` prefix** on the fact's proposition. The math-paper form `∀ x ∈ S. P` is used; multiple variables sharing a sort are comma-separated: `∀ x, y ∈ ℝ. P`. The `∈` here is binding-shorthand even when `S = Set` (as in `∀ S ∈ Set. P`); this is not a propositional membership claim.
- Other quantifiers (`∃`, nested `∀`) appear *inline* inside the proposition. Only the outermost `∀` interacts with potential future suffix sugar.
- A fact may carry side conditions with an `if` clause: `<proposition> if <condition>`. The `<condition>` is a conjunction (`∧`-separated) of atoms, where each atom is a membership (`e ∈ S`), equality (`e = e'`), or comparison (`≠`, `<`, `≤`, `>`, `≥`). Disjunction, negation, and quantifiers are not accepted; widening is deferred until a real example needs it. Widening is monotone — accepting `∨`/`¬`/quantifiers later does not invalidate any fact written under the current rule.
- **A fact is both a logical claim and a rewrite rule.** Variables bound by the outermost `∀` act as pattern variables when the fact is used as a rewrite. The kernel auto-orients facts whose sides are strictly comparable under its term order; AC marking is earned by stating commutativity and associativity (see `CLAUDE.md` for the kernel-side design).
- A `for`-suffix sugar (`P for x ∈ S`) — equivalent to wrapping the proposition with an outermost `∀` — may be added later but is not part of the core syntax.
- **Optional name.** A fact may be given a name with `fact <ident> : <proposition>`. The name is optional — most facts are auto-oriented rewrites that are never invoked by name; naming is only worth the noise when the fact will be referenced in a manual rewrite or query. The `:` parallels the sort annotation in `let name : Sort`; the parser distinguishes named from anonymous facts by lookahead for `<ident> :`. The name applies to the outer fact only — there is no syntax for labelling sub-parts of a proposition.
- **Identifier rules and namespace.** Fact names use the same identifier rules as variables and `let`-bound values, and live in the **same namespace** as `let`-bound names — a fact name shadows a value of the same name and vice versa. One symbol table, no per-keyword namespaces.

### Forms

```
fact ℚ ⊆ ℝ                                                       # subset claim, anonymous
fact 1/2 ∈ ℚ                                                     # membership claim, anonymous
fact ∀ x ∈ ℝ. x + 0 = x                                          # equality with bound vars, anonymous
fact comm_add : ∀ x, y ∈ ℝ. x + y = y + x                        # named fact
fact ∀ a, b ∈ ℝ. log(a·b) = log(a) + log(b)   if a > 0 ∧ b > 0  # with side condition
```

## Values

### Decisions so far

- Declared with `let` (see Bindings).
- **Every value belongs to a set.** The set appears as the sort annotation in the `let`. Concrete values use sets like ℝ; set-valued things use the universe `Set`.
- **Annotations are optional when there is an RHS.** A definition `let half = 1/2` is allowed; the kernel infers `ℚ` (smallest containing set; see Bindings). Annotations remain required for declarations without an RHS.
- **No function-definition sugar.** A function with a defining equation is written as a declaration plus a fact — there is no `let f(x : ℝ) : ℝ = 2·x` form.
- **No pattern arguments.** Multi-case definitions are written as multiple facts, not as pattern rows. Patterns would add no expressive power and would conflict with the "equalities are foundational" design.
- **Auto-unfolding only for literal RHS.** A `let name : T = e` definition auto-unfolds `name → e` during simplification iff `e` is a closed ground term built only from literals (numeric literals, and — once they exist — closed compositions thereof). This buys `half + half = 1` for free without erasing names whose RHS is a non-trivial expression (`let discriminant = b^2 - 4·a·c` stays folded). Mechanism: a literal-RHS `let` is treated as an auto-oriented equality; the term order makes the named symbol larger than the literal, so it orients toward the literal automatically. Non-literal-RHS `let` bindings are opaque from the kernel's perspective — to use them as a rewrite, state a separate `fact`. A `let` vs. `def` split (Lean-style) may be reintroduced later if real examples show the literal-RHS rule isn't enough.

### Forms

```
# Declared constant (opaque; characterized by later facts)
let π : ℝ
let e : ℝ

# Defined constant (with or without annotation)
let half : ℚ = 1/2
let one  : ℕ = 1
let two       = 1 + 1     # inferred ℕ
let z         = π + i     # inferred ℂ

# Declared function (a value living in a function space)
let sin : ℝ → ℝ
let exp : ℝ → ℝ

# Defined function: declaration + fact(s)
let double : ℝ → ℝ
fact ∀ x ∈ ℝ. double(x) = 2·x

let factorial : ℕ → ℕ
fact factorial(0) = 1
fact ∀ n ∈ ℕ. factorial(n+1) = (n+1) · factorial(n)
```

### Local `let` (expression-level)

Used inside an expression to bind an intermediate name. Same annotation rule: optional when the RHS is given (which is always here).

```
let r = a · a + b · b in sqrt(r)

let x = a + b in
  let y = c + d in
    x · y

let p : ℝ × ℝ = (a, b) in length(p)
```

Local `let` is at level 17 in the precedence table (binders) — its body extends rightward as far as possible.

### Anonymous functions

Lambda syntax: `(x : ℝ) ↦ body`. The parameter is annotated (parallel to the explicit-annotation rule for `let`); the codomain is computed from the body's type using the same expression-typing the kernel already performs to check `let` bindings.

```
let double : ℝ → ℝ = (x : ℝ) ↦ 2·x
let pair_sum : ℝ × ℝ → ℝ = ((x, y) : ℝ × ℝ) ↦ x + y
```

In every legal context a lambda's expected type is already known (from the surrounding `let`, function-argument signature, or fact equation), so no codomain annotation is needed on the lambda itself.

### Function arity and application

Functions take a single argument. Multi-argument functions are **uncurried** — their signatures use Cartesian products, and application uses comma-separated arguments that desugar to a tuple. Curried form (`f : ℝ → ℝ → ℝ`) is not a separate spelling for the same thing; if it appears, it denotes a different function (one returning a function).

```
let add : ℝ × ℝ → ℝ
fact ∀ x, y ∈ ℝ. add(x, y) = x + y

let dist3 : ℝ × ℝ × ℝ → ℝ
fact ∀ x, y, z ∈ ℝ. dist3(x, y, z) = sqrt(x² + y² + z²)
```

- Application: `f(x, y)` parses as `f` applied to the tuple `(x, y)`; `f(x, y)` and `f((x, y))` are the same expression.
- Tuples are first-class: `let p : ℝ × ℝ = (x, y)` then `f(p)` works.
- Cartesian product `×` is right-associative (rule chosen for consistency; tuple semantics are independent).
- Lambdas use tuple patterns: `((x, y) : ℝ × ℝ) ↦ x + y`.
- Partial application is written explicitly: `(y : ℝ) ↦ f(x, y)`.
- Nullary functions are not supported. A "constant" is just a value: `let pi : ℝ`, not `let pi : () → ℝ`.

### Subset and coercion

A value declared in a set is automatically a member of every superset (since `ℕ ⊆ ℤ ⊆ ℚ ⊆ ℝ ⊆ ℂ`). No explicit coercion is needed.

**Implicit promotion in expressions.** When an operator's operands live in different sets along a known subset chain, the kernel promotes the smaller-set operand to the larger set automatically. `2 + π` (with `2 ∈ ℕ`, `π ∈ ℝ`) is well-formed and has type `ℝ`; no `(2 : ℝ)` annotation is required. Promotion only happens along subset facts the kernel already knows (`ℕ ⊆ ℤ ⊆ ℚ ⊆ ℝ ⊆ ℂ` once the relevant facts are in scope); unrelated sets do not get implicitly bridged.

When the declared set is *narrower* than the natural one (`let small : Pos = 1/2`), the kernel must verify the membership obligation. That is a proof obligation, deferred for now.

### Overload resolution

When an operator has signatures on multiple sets (e.g., `+`, `·` defined on each of ℕ, ℤ, ℚ, ℝ, ℂ), the kernel resolves which signature applies using two rules:

**Inside-out (primary).** At each operator node, after typing the operands, find the smallest set `S` in the known subset chain that contains both operand types and for which a signature `op : S × S → _` exists. Promote both operands to `S` via implicit promotion. The result type is the codomain of that signature.

**Outside-in (tie-breaker, weak form).** If a binding annotation, function-argument signature, or fact-equation side fixes an expected type `T` for the expression, *and* inside-out yields no signature or several incomparable ones, use `T` to pick a signature whose codomain is `T` or a subset, propagate the domain back to the operands, and recurse. When inside-out succeeds unambiguously, it wins — the annotation only acts as a boundary coercion.

Worked examples (*lub* = least upper bound, i.e., the smallest set in the subset chain containing both operand types):

| Expression | Resolution | Result |
|---|---|---|
| `2 + 3` | both ℕ; `+ : ℕ × ℕ` exists | `ℕ` |
| `2 + π` | ℕ, ℝ; lub ℝ; `+ : ℝ × ℝ` exists; promote `2` | `ℝ` |
| `π + i` | ℝ, ℂ; lub ℂ; promote `π` | `ℂ` |
| `let x = 1 / 2` | ℕ, ℕ; no `/` on ℕ; walk up to ℚ | `ℚ` |
| `let x : ℝ = 1 / 2` | inside-out gives ℚ; annotation is a boundary coercion (ℚ ⊆ ℝ) | binding `ℝ`, expr `ℚ` |
| `let z : ℂ = sqrt(-1)` | inside-out ambiguous (ℝ-instance has failing obligation, ℂ-instance valid); rule 3 picks ℂ | `ℂ` |
| `(x : ℝ) ↦ 2 · x` | ℕ, ℝ; lub ℝ; promote `2`; body `ℝ` | `ℝ → ℝ` |

The "weak" form of outside-in is deliberate: typing `1/2` *as* a ℝ operation just because the binding is ℝ would be wrong for a CAS — exact ℝ arithmetic isn't generally available, and rationality is information worth preserving.

This rule covers operators whose signatures lie along the ℕ ⊆ ℤ ⊆ ℚ ⊆ ℝ ⊆ ℂ chain. **Non-chain overloading** (a `·` on matrices, polynomial rings, etc.) needs a partial-order generalization — "smallest containing set with a defined signature" becomes "most specific instance". Deferred until non-chain cases actually arise.

### Open questions

- **Narrowing proof obligations.** When a value is declared in a strict subset, how the kernel checks membership.
- **Non-chain operator overloading.** Generalizing the resolution rule beyond the ℕ–ℂ subset chain (matrices, polynomial rings, etc.). Deferred until those cases arrive.

## Queries and rewriting

### Decisions so far

- A small command layer at top level, parallel to `let` and `fact`. Commands are how the user *does* things — they don't introduce names or assert claims, they ask the kernel to compute or transform something.
- **Expression comes last.** Commands put the operation and its parameters first, the expression they act on at the end. This keeps the verb and any fact names visible at the top of a multi-line invocation, with the expression flowing below.
- Initial command set:
  - `simplify <expr>` — apply auto-oriented rewrites, AC normalization, identity-element absorption, and literal arithmetic to a fixed point.
  - `apply <name> to <expr>` — single manual rewrite step using a named fact.
  - `evaluate <expr>` — literal arithmetic on ℕ/ℤ/ℚ only; no rewrites fire.
  - `prove <prop>` — placeholder; deferred until a proof story exists.
- **Direction of manual rewriting.** `apply <name> to <expr>` uses the fact's as-written orientation (LHS pattern, RHS replacement). `apply ← <name> to <expr>` flips it (RHS pattern, LHS replacement). The `←` is placed before the name so it reads "apply the reverse of `<name>`".
  - For auto-oriented facts (sides strictly comparable), `apply` re-fires the canonical direction; `apply ←` is the only way to invoke the reverse.
  - For incomparable equalities (factor/expand pairs, etc.), neither direction is canonical; the user picks per call.
- **Naming requirement.** `apply` requires a named fact — anonymous facts can only fire via `simplify`. This matches the "name a fact only when you'll invoke it manually" rule under Facts.

### Forms

```
simplify (x + 0) · (y + y)

apply comm_add to x + y

apply ← log_product to
    log(x · y · z) + log(w)

evaluate 2^10 + 3·5
```

### Open questions

- **Localizing a rewrite.** Whether `apply` grows an `at <path>` clause to target a subterm, or whether a separate `rewrite … in … at …` form is needed. Deferred until examples demand it.
- **Composing commands.** Whether commands chain (`apply f1 to e |> apply f2`) or whether multi-step rewrites are written as a sequence of `let`-bound intermediates. Deferred.
- **REPL vs. file form.** Whether the same command syntax is used at a REPL prompt and inside a file, or whether the REPL gets a terser prefix. Deferred.

## Term order

### Decisions so far

- **The kernel uses Knuth–Bendix Order (KBO)** as its well-founded term order for auto-orientation. Each symbol has a non-negative weight; comparison is by total weight first, then by precedence on the head, then lexicographically on arguments. This aligns "smaller" with "fewer symbols," matching the user's intuition of simpler. Equalities whose two sides are KBO-incomparable (e.g., distributivity) remain user-invoked.
- **Per-symbol weights.** Each symbol carries a weight, default `1`, settable at the symbol's declaration site. Variables share a single fixed weight `w₀ = 1`. KBO admissibility allows at most one symbol of weight 0, which must be unary and maximal in precedence; deferred until a use case appears.
- **Precedence is declared once per module in a `precedence` block.** The block lists symbols in increasing precedence order using `<`. Multiple modules may each contribute a fragment; the kernel assembles a single global precedence by merging the fragments. Inconsistent constraints across modules are an error.

```
precedence: + < · < ^ < f < g
```

- **Why a block, not per-symbol numeric annotations.** Precedence is inherently relative; absolute numbers force gap-and-renumber discipline and scatter the global picture across many sites. A single block keeps the order visible in one place and maps directly to KBO's mathematical definition (a strict order on symbols).
- **Why not implicit declaration order.** Reordering declarations would silently change auto-orientation, and cross-file imports would make the global order fragile.

### Open questions

- **Cross-module merge semantics.** Exact behavior when two modules' precedence fragments conflict (hard error vs. require explicit re-statement at the import site). Deferred until multi-file examples exist.
- **Weight-0 unary symbol.** Whether to expose KBO's allowance for a single weight-0 unary symbol (e.g., for negation or a "free" wrapper). Deferred.
- **AC-KBO.** AC operators are flattened and sorted before comparison; the exact AC-KBO variant used (and how operand multiset comparison interacts with the lex tiebreak) is deferred to the kernel-implementation phase.

## AC recognition

### Decisions so far

- **Pattern recognition is syntactic, up to obvious normalization.** A fact is recognized as commutativity for `f` if it has the shape `∀ <vars>. f(a, b) = f(b, a)` with `a` and `b` distinct bound variables (α-renaming irrelevant). A fact is recognized as associativity for `f` if it has shape `∀ <vars>. f(f(a, b), c) = f(a, f(b, c))` *or* its mirror `∀ <vars>. f(a, f(b, c)) = f(f(a, b), c)`, with `a`, `b`, `c` distinct bound variables. Side conditions (`if …`) on the fact disqualify it from AC recognition. The kernel does not attempt to prove that an arbitrary fact is logically equivalent to AC — recognition is a syntactic gate, not a semantic one.
- **Partial AC is tracked with independent flags.** Each operator carries two flags, `commutative` and `associative`, set independently as the corresponding facts are read.
  - **Associative-only:** applications are flattened to n-ary form; operand order is preserved. Useful for non-commutative operators with associative concat-like behavior (string concatenation, matrix multiplication, function composition).
  - **Commutative-only:** at fixed binary arity, the two operands are sorted by the kernel's term order. No flattening.
  - **Both (AC):** flatten to n-ary, then sort operands. This is the case described in `CLAUDE.md`.
  - Identity-element marking (`CLAUDE.md`) layers on top of whichever flag is set: a left/right identity collapses operands of a flattened (associative or AC) application; for a commutative-only operator, identity rewriting still fires as an auto-oriented rewrite but without n-ary collapse.
- **Marking is per-(symbol, set).** `fact ∀ x, y ∈ ℝ. x + y = y + x` marks `+` commutative *on ℝ*, not on `+` globally. An application `a + b` is treated as commutative only when both operands' types are subsets of the set `S` over which the AC fact was stated. To get AC on a wider set the user states the fact again at that set; a separate fact relating the two signatures is what would license lifting, and that machinery is not provided.
  - Consequence: along the ℕ ⊆ ℤ ⊆ ℚ ⊆ ℝ ⊆ ℂ chain, commutativity and associativity for arithmetic operators must be stated at the widest set used in practice (typically ℂ) and rely on implicit promotion to bring narrower operands up before the operator applies. A library may state them once at ℂ and once at any narrower set whose closure under the operator the user wants to reason about without promotion.

### Open questions

- **Lifting AC marks along subset chains.** Whether to grow a mechanism that propagates an AC mark from `S` to `T` when `S ⊆ T` and the operator's signatures on `S` and `T` are known to agree on `S`. Deferred until a concrete example shows the per-set restatement is painful.
- **Recognizing AC up to AC.** Once `+` is AC-marked, a later fact like `∀ a, b, c. a + b + c = c + b + a` is provable (by AC) but not in the canonical commutativity shape. Whether such facts should be silently accepted as redundant or rejected is open.

## Other syntax topics

(Pending: file structure, variable binding form for facts.)
