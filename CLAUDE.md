# CLAUDE.md

## Project Overview

**regler** is a Computer Algebra System (CAS) built around a minimal kernel.

## Design decisions

- **Equalities are the foundation.** Equalities (symmetric mathematical claims, possibly with side conditions) are the primitive. Rewriting is a derived operation that uses an equality together with a direction, location, and any conditions. Rewrite rules are not a separate kind of object — they are uses of equalities.
- **Built-in number sets and literals.** ℕ, ℤ, and ℚ are built in, with arbitrary-precision arithmetic. Integer and rational literals are supported. Decimals and floating-point numbers are not supported.
- **ℝ and ℂ are not kernel primitives.** They are library-defined sets, axiomatized via equalities (e.g., `ℚ ⊆ ℝ`, `i² = -1`). Constants like π, e, i and functions like sin, exp, sqrt are introduced with characterizing equalities, not built into the kernel.
- **General set machinery in the kernel.** The kernel provides sets/types as first-class objects, subset relations, membership reasoning, function signatures over sets, and reasoning under assumptions. ℕ/ℤ/ℚ use this machinery and are additionally backed by built-in data; ℝ/ℂ use the same machinery with only axioms.
- **Operators start naked.** Beyond literal arithmetic on ℕ/ℤ/ℚ (which the kernel evaluates directly), the kernel knows nothing about operators like `+`, `·`, `∪`, `∩`. Properties such as commutativity, associativity, identity elements, and distributivity must be stated as facts.
- **AC marking is earned dynamically.** When the kernel reads a fact whose shape matches commutativity (`∀ a, b. f(a, b) = f(b, a)`) and one matching associativity for the same operator, that operator is promoted to AC representation: applications are flattened and operands sorted, so commutativity and associativity hold by construction thereafter.
- **Identity-element marking is earned similarly.** When the kernel reads a fact of shape `∀ x ∈ S. f(x, e) = x` (right identity) or `∀ x ∈ S. f(e, x) = x` (left identity), where `e` is a closed term, `e` is marked as the corresponding-side identity for `f`. During normalization, `f(…, e, …)` collapses by dropping `e` operands. If `f` is AC-marked, left and right coincide and one fact suffices; otherwise the two sides are tracked separately. The marking is on top of the auto-oriented rewrite — it lets `e` be absorbed directly from flattened AC applications instead of re-firing the rewrite at every step.
- **Auto-orientation by term order.** The kernel has a fixed well-founded term order. Any equality whose two sides are strictly comparable under this order is auto-oriented toward the smaller side and applied automatically during simplification — the user does not need to invoke it. Equalities whose sides are incomparable (e.g., factor/expand pairs) remain user-invoked.
- **Fact = logical claim + rewrite rule.** A `fact` serves both roles. Variables bound by the outermost `∀` act as pattern variables when the fact is used as a rewrite. There is no separate "rule" concept.

## Coding conventions

- **Consult `docs/syntax-notes.md` first.** Before making any design or implementation decision — new syntax, operator precedence, AST nodes, surface forms, fact shapes, command syntax — read `docs/syntax-notes.md`. It records the authoritative decisions and open questions for the concrete syntax. Do not invent or guess syntax rules that may already be decided there.
- **Document function purpose.** Every function should have a comment describing its purpose. Exception: small functions whose purpose is self-evident from the name and signature.

## Working notes

- `docs/syntax-notes.md` — concrete syntax discussion (tentative). Currently covers bindings, sets, facts, values.
- `docs/syntax-open-questions.md` — checklist of syntax decisions still to make.
- `docs/changelog.md` — per-version log of program changes, keyed to the version in `Cargo.toml`. Each version bump adds a section describing what changed in that release.
- `examples/` — surface-syntax sketches exercising the design against the long-term goals. Work-in-progress, not type-checked; expect gaps and inconsistencies.

## Milestones

The strategy is to get a working end-to-end spine early, then deepen iteratively. Every milestone from 2 onward adds at least one example file that the implementation actually runs — validation-by-examples is a continuous practice, not a standalone phase.

### Completed

1. **Choose implementation language and parser approach.** Informed by kernel needs (arbitrary-precision arithmetic, ADTs for terms, Unicode). Hand-written recursive descent is the default for the parser unless a strong reason to use a generator emerges.
2. **Round-trip REPL on a minimal surface.** Identifiers, integer literals, `+ · ^ = ( )`, `let name = expr`, `fact <expr>`, and a `print` command. Lexer → parser → AST → pretty-printer, with the property `parse(print(t)) == t`. No kernel yet.
3. **Kernel term representation and `evaluate`.** Internal uniform-prefix terms, substitution, literal arithmetic on ℕ. Adds the `evaluate` command.
4. **Auto-oriented rewriting.** KBO with default weights = 1; one user-stated equality fires as a rewrite via `simplify`. No AC, no side conditions.
5. **AC recognition and identity-element marking.** Earn the marks from facts as designed; flatten and sort AC applications.
6. **Widen numeric tower.** ℤ, ℚ, the subset chain, implicit promotion.
7. **Side conditions on facts** (`if` clauses), then `apply` and `apply ←`.
8. **Sets as first-class.** Membership, subset, set-builder — introduced when the first example genuinely needs them (likely with `sin`, `Pos`, etc.).

### Upcoming

9. **Partial AC normalization.** The `saw_comm` and `saw_assoc` flags are already tracked independently, but only the fully-promoted AC case drives normalization. This milestone wires up the partial behaviors: associativity-only flattens nested applications to n-ary form (order preserved); commutativity-only sorts the two arguments of a binary application by the kernel's term order. Both compose correctly with identity-element dropping. No new surface syntax. Adds tests for assoc-only (e.g. `∘`), comm-only, and identity interactions in both partial cases.

10. **Complex numbers.** No new language features — uses only the existing set and rewriting machinery. Adds `ℂ : Set`, `fact ℝ ⊆ ℂ`, `let i : ℂ`, `fact i·i = -1`, and commutativity/associativity for `+` and `·` on ℂ. New runnable example `examples/complex.rgl` demonstrating: `simplify (1 + i)·(1 - i)` → `2`, `simplify i^4` → `1`.

11. **Multi-line input and imports.** Files are parsed statement-by-statement using indentation-based continuation (a non-empty line whose indent exceeds its statement's first line is a continuation). `import "path.rgl"` loads a file relative to the importing file, brings its names into the flat namespace, and is idempotent; cycles are an error. The REPL keeps its line-at-a-time behaviour. Adds standard library files (e.g. `lib/complex.rgl`) that subsequent examples can import.

12. **Lambda expressions and beta reduction.** `(x : T) ↦ body` is added to the AST, parser, and printer. Beta reduction fires in `simplify`. Pattern matching in facts can match lambda-headed patterns, enabling higher-order operators like `D` and `∫` to be defined by rewrite rules. Open question: scope of this milestone (lambda as definition RHS vs. lambda only in facts).

13. **Derivatives.** `D` declared as `(ℝ → ℝ) → (ℝ → ℝ)`. Facts for: constant rule, identity rule, linearity, product rule, chain rule, and known derivatives (`sin`, `cos`, `exp`, `log`). `simplify` computes symbolic derivatives of polynomial and composed expressions. New runnable example `examples/deriv.rgl`.

14. **Polynomial roots.** Introduces `√` as a declared function with characterizing fact (`√(x)^2 = x if x ≥ 0`). Handles linear and quadratic equations. Open question: new `solve` command vs. purely fact-and-apply approach; representation of two roots (± notation or set of roots). New runnable example `examples/roots.rgl`.

15. **Indefinite integrals.** `∫` declared as `(ℝ → ℝ) → (ℝ → ℝ)`. Facts for: power rule, linearity, and integrals of basic trigonometric functions. `simplify` computes indefinite integrals of polynomial and simple trigonometric expressions. New runnable example `examples/int.rgl`.

16. **Onward, example-driven.** ODEs and further goals once differentiation and integration are solid. Each new long-term goal drives the next surface/kernel additions.

Deferred indefinitely (no milestone slot until an example demands it): set-polymorphic value declarations, user-defined infix operators, theorem/proof syntax, ASCII fallbacks, qualified imports.

## Long-term goals

- Support complex numbers
- Compute derivatives of functions
- Find roots of some polynomial equations
- Find some definite and indefinite integrals
- Solve some ordinary differential equations