# Syntax: open questions and deferred decisions

Items still to resolve. When an item is settled, remove it here and record the decision in `docs/syntax.md`.

Grouped by how soon each one blocks writing realistic example files.

## Needed before the next relevant example

- **Set-polymorphic value declarations.** Surfaced by writing `examples/fun.rgl` (function composition `∘` and `id`). The natural signature `(B → C) × (A → B) → (A → C)` is parametric in three sets, but no current syntax expresses a value whose *type* is parameterized by Set arguments. Three candidate surface forms:

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
     No new binder kind — just dependent `→`. Uses become noisy: `∘(ℝ, ℝ, ℝ)(f, g)` unless paired with implicit-argument elision.

  4. **Status quo: monomorphic restatement.** State `∘` and `id` per concrete set. No new machinery. Verbose if many function spaces are used in one file.

  Cross-cutting questions any of (1)–(3) must answer:
  - **AC recognition past Set-binders.** With set-polymorphism the associativity fact has an outer Set-∀ and an inner value-∀; recognition must treat the Set prefix as transparent (or be extended explicitly). Same question for per-(symbol, set) marking.
  - **Identity-element recognition for polymorphic `id`.** `CLAUDE.md` requires the identity element `e` to be a closed term. Polymorphic `id` is closed only up to Set-polymorphism; the rule must be widened, or `id` won't earn identity-element marking.
  - **Term order / KBO.** Whether the Set arguments of a polymorphic symbol contribute to its weight, or are invisible to KBO.

## Important, deferrable until needed

- **Localizing a rewrite.** Whether `apply` grows an `at <path>` clause to target a specific subterm, or whether a separate `rewrite … in … at …` form is needed.
- **Composing commands.** Whether commands chain (`apply f1 to e |> apply f2`) or whether multi-step rewrites are written as a sequence of `let`-bound intermediates.
- **REPL vs. file form.** Whether the same command syntax is used at a REPL prompt and inside a file, or whether the REPL gets a terser prefix.
- **Superscript powers.** Whether `x²` is accepted as sugar for `x^2` (depends on identifier rules; may conflict with superscripts-as-identifier-characters).
- **Inline `if then else`.** Listed at precedence level 18 but not yet implemented; conditional behavior can be encoded via separate facts with `if` side conditions for now.
- **Lifting AC marks along subset chains.** Whether to grow a mechanism that propagates an AC mark from `S` to `T` when `S ⊆ T` and the operator's signatures on `S` and `T` are known to agree on `S`. Deferred until the per-set restatement becomes painful.
- **Recognizing AC up to AC.** Once `+` is AC-marked, a later fact like `∀ a, b, c. a + b + c = c + b + a` is provable by AC but not in the canonical commutativity shape. Whether such facts should be silently accepted as redundant or rejected is open.
- **Cross-module precedence merge semantics.** Exact behavior when two modules' precedence fragments conflict: hard error vs. require explicit re-statement at the import site. Deferred until multi-file examples exist.
- **Non-chain operator overloading.** Generalizing the overload-resolution rule beyond the ℕ–ℂ subset chain (matrices, polynomial rings, etc.) to a partial-order of sets with a "most specific instance" rule. Deferred until those cases arrive.

## Deferrable

- **Theorem keyword and proof syntax.** Not needed until proofs are written.
- **User-configurable infix operators.** A user-defined symbol cannot currently be declared as infix; the fixity table is fixed at parser-build time. Open: an `infix <prec> <assoc>` declaration form (or similar), how it interacts with the per-module precedence block, and whether prefix-form use of an infix symbol (`+(a, b)`) is also accepted.
- **ASCII fallbacks.** Whether `in`, `subset`, `forall`, etc. are accepted alongside Unicode forms.
- **Sort hierarchy beyond `Set`.** Whether a higher universe is ever needed.
- **String literals, printing, I/O.** Only needed for runnable examples with output.
- **Pretty-printing rules.** Line width, parenthesization choices for display.
- **Qualified imports.** `import "foo.reg" as Foo` introducing a `Foo.bar` namespace. Would add a second namespace layer; deferred until flat-namespace collisions become painful.
- **Selective imports.** `import {sin, cos} from "trig.reg"`.
- **Standard-library search path.** `import std/arith` as a second form alongside relative-path imports.
- **Weight-0 unary symbol in KBO.** Whether to expose KBO's allowance for a single weight-0 unary symbol (e.g., for negation or a "free" wrapper).
- **AC-KBO variant.** AC operators are flattened and sorted before comparison; the exact AC-KBO variant (and how operand multiset comparison interacts with the lex tiebreak) is deferred to the kernel-implementation phase.

## Cross-cutting

- **Declaration-then-fact verbosity.** Verbose for long subset chains (ℕ ⊆ ℤ ⊆ ℚ ⊆ ℝ ⊆ ℂ requires 4 separate facts) and for parameterized sets. Acceptable for now; revisit if it becomes painful in real examples. The flat-namespace import tradeoff has the same character — simple rule, potential pain at scale.
- **Whether to reintroduce definition sugar.** Function-definition sugar (`let f(x : ℝ) : ℝ = 2·x`) and parameterized-set definition sugar were rejected. Revisit if their absence hurts readability in real examples.
