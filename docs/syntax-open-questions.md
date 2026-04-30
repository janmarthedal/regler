# Syntax open questions

Checklist of syntax decisions still to make. Update by removing items as they are settled (and recording the decision in `syntax-notes.md`).

Grouped by how soon each one blocks writing realistic example files.

## Blocks coherent example files

(none — the syntax surface is settled enough to start writing example files)

## Important, can be sketched after first examples

- [ ] **Set-polymorphic value declarations.** Surfaced by writing `examples/fun.rgl` (function composition `∘` and `id`). The natural signature `(B → C) × (A → B) → (A → C)` is parametric in three sets, but no current syntax expresses a value whose *type* is parameterized by Set arguments. Three candidate surface forms:

  1. **Prenex ∀ in type annotations.**
     ```
     let ∘  : ∀ A, B, C ∈ Set. (B → C) × (A → B) → (A → C)
     let id : ∀ A ∈ Set. A → A
     ```
     New binder kind in type position; closer to System F / ML let-polymorphism. Set arguments inferred at each use by unifying against operand types.

  2. **Implicit arguments.**
     ```
     let ∘  : {A, B, C : Set} → (B → C) × (A → B) → (A → C)
     let id : {A : Set} → A → A
     ```
     Same semantics as (1); different surface signal that the arguments are inferred rather than written.

  3. **Explicit dependent function.**
     ```
     let ∘  : (A B C : Set) → (B → C) × (A → B) → (A → C)
     let id : (A : Set) → A → A
     ```
     No new binder kind — just dependent `→`, which is a smaller extension of the current `→` than (1)/(2). Uses become noisy: `∘(ℝ, ℝ, ℝ)(f, g)` unless paired with implicit-argument elision.

  4. **Status quo: monomorphic restatement.** State ∘ and `id` per concrete set (e.g. on `ℝ → ℝ`). No new machinery, matches the per-(symbol, set) AC-marking rule. Verbose if many function spaces are used in a single file.

  Cross-cutting questions any of (1)–(3) must answer:
  - **AC recognition past Set-binders.** `syntax-notes.md:463` matches `∀ <vars>. f(f(a,b),c) = f(a,f(b,c))` syntactically. With set-polymorphism the associativity fact has an outer Set-∀ and an inner value-∀; recognition must treat the Set prefix as transparent (or be extended explicitly). Same question for the per-(symbol, set) marking — what counts as "the set" when the operator's signature is itself polymorphic?
  - **Identity-element recognition for polymorphic `id`.** `CLAUDE.md` requires the identity element `e` to be a closed term. Polymorphic `id` is closed only up to Set-polymorphism; the rule must be widened, or `id` won't earn identity-element marking and `f ∘ id = f` will only fire as an ordinary (KBO-incomparable) rewrite.
  - **Term order / KBO.** Whether the Set arguments of a polymorphic symbol contribute to its weight, or are invisible to KBO (likely the latter, but worth stating).

## Deferrable

- [ ] **Theorem keyword and proof syntax.** Not needed until proofs are written.
- [ ] **User-configurable infix operators.** A user-defined symbol cannot currently be declared as infix; the fixity table in `syntax-notes.md` is fixed at parser-build time. Open: a `infix <prec> <assoc>` declaration form (or similar), how it interacts with the per-module precedence block, and whether prefix-form use of an infix symbol (`+(a, b)`) is also accepted.
- [ ] **ASCII fallbacks.** Whether `in`, `subset`, `forall`, etc. are accepted alongside Unicode.
- [ ] **Sort hierarchy beyond `Set`.** Whether a higher universe is ever needed.
- [ ] **String literals, printing, I/O.** Only needed for runnable examples.
- [ ] **Pretty-printing rules.**

## Cross-cutting

- [ ] **The declaration-then-fact pattern's verbosity.** Verbose for long subset chains and parameterized sets. Acceptable for now; revisit if it becomes painful.
- [ ] **Whether to reintroduce sugar.** Function-definition sugar (form-4) and parameterized-set definition sugar were rejected. Keep an eye out for cases where their absence hurts readability badly.
