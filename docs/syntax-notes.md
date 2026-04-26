# Syntax notes

Working notes on concrete syntax. Decisions here are tentative — recorded so the design conversation can resume later without rederiving everything.

## Sets

### Decisions so far

- Keyword: `set` (single keyword for both declarations and definitions; `type` and "structure" are deferred).
- **First-class but bounded.** Sets are values: they can be named, passed as arguments, returned from functions. But there is no general set theory: a fixed vocabulary of operations (`∪`, `∩`, `\`, `×`, `→`, set-builder) is provided, and `Set` is a universe (sort), not itself a set. You cannot write `Set ∈ Set`.
- **No declaration-time constraint sugar.** `set ℝ ⊇ ℚ` is *not* allowed. The verbose form `set ℝ; axiom ℚ ⊆ ℝ` is required. This keeps declarations and axioms cleanly separated.
- **Six forms of set declaration/definition** are supported (see below).

### The six forms

```
-- 1. Bare opaque declaration
set ℝ

-- 2. Declaration plus axiomatic constraints (separate statements)
set ℝ
axiom ℚ ⊆ ℝ

set ℂ
axiom ℝ ⊆ ℂ

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

-- Subset claims (in axioms or theorems)
axiom ℕ ⊆ ℤ
axiom ℤ ⊆ ℚ

-- Function signatures (sets as domain/codomain)
f   : ℝ → ℝ
g   : ℝ × ℝ → ℝ
sin : ℝ → Interval(-1, 1)
recip : Nonzero → ℝ

-- Inline (anonymous) sets inside a signature
abs_inv : {x ∈ ℝ | x ≠ 0} → Pos

-- Variable annotations (placeholder syntax; binding form is open)
axiom (x : ℝ)    ⊢ x + 0 = x
axiom (x, y : ℝ) ⊢ x + y = y + x

-- Side conditions
axiom (a, b : ℝ) ⊢ log(a·b) = log(a) + log(b)   if a ∈ Pos ∧ b ∈ Pos

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

- **Binding/quantifier syntax in axioms.** Used `(x : ℝ) ⊢ ...` as placeholder. Alternative: implicit universal quantification over free variables, with sorts declared separately (`var x : ℝ`) or inferred. Needs to be settled before writing real example files.
- **Condition language.** `if` clauses currently allow conjunctions of membership, equality, inequality. Whether richer logic is permitted (disjunction, negation, quantifiers) is open.
- **The declaration-then-axiom pattern.** Works but verbose for long subset chains (ℕ ⊆ ℤ ⊆ ℚ ⊆ ℝ ⊆ ℂ requires 4 separate axioms). Acceptable for now; revisit if it becomes painful in real examples.
- **ASCII fallbacks.** Whether `in`, `subset`, `forall`, etc. are accepted alongside the Unicode forms — deferred.
- **Sort of `Set`.** Treated as a universe (you can't write `ℝ ∈ Set` as a proposition, only `S : Set` as a parameter declaration). Whether the language ever needs a higher universe (a "sort of Set") is deferred — not needed for current goals.

## Other syntax topics

(Pending: equalities and rewriting, function declarations, file structure, queries.)
