# Syntax notes

Working notes on concrete syntax. Decisions here are tentative — recorded so the design conversation can resume later without rederiving everything.

## Bindings (`let`)

A single keyword `let` introduces every named thing — values, functions, and sets. Declarations are distinguished from definitions by the presence of `=`:

- `let name : Sort` — declaration (opaque; characterized later by facts).
- `let name : Sort = Expr` — definition.

Every `let` carries an explicit sort/type annotation; there is no inference. The annotation is the set the value belongs to (ℕ, ℝ, ℝ → ℝ, …) or the universe `Set` for set-valued things.

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
- A fact may carry side conditions with an `if` clause: `<proposition> if <condition>`.
- A `for`-suffix sugar (`P for x ∈ S`) — equivalent to wrapping the proposition with an outermost `∀` — may be added later but is not part of the core syntax.

### Forms

```
fact ℚ ⊆ ℝ                                                       # subset claim
fact 1/2 ∈ ℚ                                                     # membership claim
fact ∀ x ∈ ℝ. x + 0 = x                                      # equality with bound vars
fact ∀ a, b ∈ ℝ. log(a·b) = log(a) + log(b)   if a > 0 ∧ b > 0  # with side condition
```

### Open questions

- **Condition language.** `if` clauses currently allow conjunctions of membership, equality, inequality. Whether richer logic is permitted (disjunction, negation, quantifiers) is open.

## Values

### Decisions so far

- Declared with `let` (see Bindings).
- **Every value belongs to a set.** The set appears as the sort annotation in the `let`. Concrete values use sets like ℝ; set-valued things use the universe `Set`.
- **Explicit annotations are required.** No inference, even when the RHS makes the set obvious. (`let half : ℚ = 1/2`, never `let half = 1/2`.)
- **No function-definition sugar.** A function with a defining equation is written as a declaration plus a fact — there is no `let f(x : ℝ) : ℝ = 2·x` form.
- **No pattern arguments.** Multi-case definitions are written as multiple facts, not as pattern rows. Patterns would add no expressive power and would conflict with the "equalities are foundational" design.

### Forms

```
# Declared constant (opaque; characterized by later facts)
let π : ℝ
let e : ℝ

# Defined constant
let half : ℚ = 1/2
let one  : ℕ = 1

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

When the declared set is *narrower* than the natural one (`let small : Pos = 1/2`), the kernel must verify the membership obligation. That is a proof obligation, deferred for now.

### Open questions

- **Narrowing proof obligations.** When a value is declared in a strict subset, how the kernel checks membership.
- **Overloaded operators in lambda bodies.** When the body uses operators like `·` whose signatures exist on multiple sets, codomain inference may need a tie-breaking rule (e.g., smallest containing set, or require explicit annotation).

## Other syntax topics

(Pending: rewriting/queries, file structure, variable binding form for facts.)
