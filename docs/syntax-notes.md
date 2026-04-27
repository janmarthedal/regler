# Syntax notes

Working notes on concrete syntax. Decisions here are tentative — recorded so the design conversation can resume later without rederiving everything.

## Sets

### Decisions so far

- Keyword: `set` (single keyword for both declarations and definitions; `type` and "structure" are deferred).
- **First-class but bounded.** Sets are values: they can be named, passed as arguments, returned from functions. But there is no general set theory: a fixed vocabulary of operations (`∪`, `∩`, `\`, `×`, `→`, set-builder) is provided, and `Set` is a universe (sort), not itself a set. You cannot write `Set ∈ Set`.
- **No declaration-time constraint sugar.** `set ℝ ⊇ ℚ` is *not* allowed. The verbose form `set ℝ; fact ℚ ⊆ ℝ` is required. This keeps declarations and facts cleanly separated.
- **Six forms of set declaration/definition** are supported (see below).

### The six forms

```
-- 1. Bare opaque declaration
set ℝ

-- 2. Declaration plus separate fact statements
set ℝ
fact ℚ ⊆ ℝ

set ℂ
fact ℝ ⊆ ℂ

-- 3. Definition by enumeration (extensional)
set Digits = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9}
set Bit    = {0, 1}

-- 4. Definition by predicate (subset comprehension)
set Pos     = {x ∈ ℝ | x > 0}
set Nonzero = {x ∈ ℝ | x ≠ 0}

-- 5. Definition by image
set Squares = {n² | n ∈ ℕ}
set Evens   = {2·k | k ∈ ℤ}

-- 6. Image with filter (combined)
set EvenSquares = {n² | n ∈ ℕ, n mod 2 = 0}

-- 7. Definition by set algebra
set NonzeroReals = ℝ \ {0}
set ℚ⁺           = ℚ ∩ Pos
set RealPairs    = ℝ × ℝ
set RealEndo     = ℝ → ℝ

-- 8. Parameterized set (a function returning a set)
set Interval(a : ℝ, b : ℝ)     = {x ∈ ℝ | a ≤ x ∧ x ≤ b}
set OpenInterval(a : ℝ, b : ℝ) = {x ∈ ℝ | a < x ∧ x < b}
set Multiples(n : ℤ)           = {n·k | k ∈ ℤ}

-- 9. Parameterized over a set (Set as a sort/universe)
set Pairs(S : Set, T : Set)    = S × T
set Endo(S : Set)              = S → S
```

(Numbered 1–9 above for ease of reference; "six forms" refers to the conceptual categories: opaque declaration, extensional, predicate-subset, image, set-algebra, parameterized.)

### Usage examples

```
-- Membership claims
1/2 ∈ ℚ
π   ∈ ℝ \ ℚ
0   ∈ Bit

-- Subset claims (in facts or theorems)
fact ℕ ⊆ ℤ
fact ℤ ⊆ ℚ

-- Function signatures (sets as domain/codomain)
f   : ℝ → ℝ
g   : ℝ × ℝ → ℝ
sin : ℝ → Interval(-1, 1)
recip : Nonzero → ℝ

-- Inline (anonymous) sets inside a signature
abs_inv : {x ∈ ℝ | x ≠ 0} → Pos

-- Variable annotations (placeholder syntax; binding form is open)
fact (x : ℝ)    ⊢ x + 0 = x
fact (x, y : ℝ) ⊢ x + y = y + x

-- Side conditions
fact (a, b : ℝ) ⊢ log(a·b) = log(a) + log(b)   if a ∈ Pos ∧ b ∈ Pos

-- Parameterized sets used like any function call
let UnitInterval = Interval(0, 1)
clamp : ℝ → Interval(0, 1)
m     : Endo(ℝ)

-- Set algebra inline
to_rat      : ℚ ∩ Pos → ℚ
union_check : ℕ ∪ {-1, -2} → ℤ

-- Set-builder used directly without naming
sum_over : {n ∈ ℕ | n ≤ 10} → ℕ
```

### Open questions

- **The declaration-then-fact pattern.** Works but verbose for long subset chains (ℕ ⊆ ℤ ⊆ ℚ ⊆ ℝ ⊆ ℂ requires 4 separate facts). Acceptable for now; revisit if it becomes painful in real examples.
- **ASCII fallbacks.** Whether `in`, `subset`, `forall`, etc. are accepted alongside the Unicode forms — deferred.
- **Sort of `Set`.** Treated as a universe (you can't write `ℝ ∈ Set` as a proposition, only `S : Set` as a parameter declaration). Whether the language ever needs a higher universe (a "sort of Set") is deferred — not needed for current goals.

## Facts

### Decisions so far

- Keyword: `fact`. Used to assert any statement the system should treat as given — equalities, subset claims, membership claims, and the defining equations of declared functions.
- **One keyword for all asserted statements.** The syntax does not distinguish "axioms" (taken as fundamental) from "definitions" (introducing meaning); both are facts the kernel is told. A future `theorem` keyword may be added for proved statements.
- A fact may bind variables (`(x : ℝ) ⊢ ...`) and carry side conditions (`if P`).

### Forms

```
fact ℚ ⊆ ℝ                                                       -- subset claim
fact 1/2 ∈ ℚ                                                     -- membership claim
fact (x : ℝ)    ⊢ x + 0 = x                                      -- equality with bound vars
fact (a, b : ℝ) ⊢ log(a·b) = log(a) + log(b)   if a > 0 ∧ b > 0  -- with side condition
```

### Open questions

- **Binding/quantifier syntax.** `(x : ℝ) ⊢ ...` is placeholder. Alternative: implicit universal quantification over free variables, with sorts declared separately or inferred. Needs to be settled before writing real example files.
- **Condition language.** `if` clauses currently allow conjunctions of membership, equality, inequality. Whether richer logic is permitted (disjunction, negation, quantifiers) is open.

## Values

### Decisions so far

- Keyword: `let`. Used for both declarations and definitions, distinguished by presence of `=` (parallel to `set`).
- **Every value belongs to a set.** A type annotation is part of every `let`. Sets themselves are not values in this sense — they live in the universe `Set` and use the `set` keyword.
- **Explicit type annotations are required.** No inference, even when the RHS makes the set obvious. (`let half : ℚ = 1/2`, never `let half = 1/2`.)
- **No function-definition sugar.** A function with a defining equation is written as a declaration plus a fact — there is no `let f(x : ℝ) : ℝ = 2·x` form.
- **No pattern arguments.** Multi-case definitions are written as multiple facts, not as pattern rows. Patterns would add no expressive power and would conflict with the "equalities are foundational" design.

### Forms

```
-- Declared constant (opaque; characterized by later facts)
let π : ℝ
let e : ℝ

-- Defined constant
let half : ℚ = 1/2
let one  : ℕ = 1

-- Declared function (a value living in a function space)
let sin : ℝ → ℝ
let exp : ℝ → ℝ

-- Defined function: declaration + fact(s)
let double : ℝ → ℝ
fact (x : ℝ) ⊢ double(x) = 2·x

let factorial : ℕ → ℕ
fact factorial(0) = 1
fact (n : ℕ) ⊢ factorial(n+1) = (n+1) · factorial(n)
```

### Subset and coercion

A value declared in a set is automatically a member of every superset (since `ℕ ⊆ ℤ ⊆ ℚ ⊆ ℝ ⊆ ℂ`). No explicit coercion is needed.

When the declared set is *narrower* than the natural one (`let small : Pos = 1/2`), the kernel must verify the membership obligation. That is a proof obligation, deferred for now.

### Open questions

- **Lambda / anonymous functions.** Whether expressions like `λ x : ℝ. 2·x` or `(x : ℝ) ↦ 2·x` are needed, or whether named functions suffice for everything a CAS needs to express.
- **Function arity / currying.** Whether `f : ℝ × ℝ → ℝ` (one pair argument) or `f : ℝ → ℝ → ℝ` (curried) is the default, or whether both are allowed.
- **Narrowing proof obligations.** When a value is declared in a strict subset, how the kernel checks membership.

## Other syntax topics

(Pending: rewriting/queries, file structure, variable binding form for facts.)
