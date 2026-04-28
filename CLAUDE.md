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
- **Auto-orientation by term order.** The kernel has a fixed well-founded term order. Any equality whose two sides are strictly comparable under this order is auto-oriented toward the smaller side and applied automatically during simplification — the user does not need to invoke it. Equalities whose sides are incomparable (e.g., factor/expand pairs) remain user-invoked.
- **Fact = logical claim + rewrite rule.** A `fact` serves both roles. Variables bound by the outermost `∀` act as pattern variables when the fact is used as a rewrite. There is no separate "rule" concept.

## Working notes

- `docs/syntax-notes.md` — concrete syntax discussion (tentative). Currently covers bindings, sets, facts, values.
- `docs/syntax-open-questions.md` — checklist of syntax decisions still to make.

## Milestones

1. Design the surface syntax and validate it by writing example files that express each of the long-term goals.
2. Distill the kernel built-ins and core algorithms implied by the syntax.
3. Choose a restricted POC scope: simplify a univariate polynomial.
4. Decide on the implementation language, informed by the kernel's algorithmic needs.
5. Build lexer, parser, kernel, and REPL covering the POC scope.
6. Extend and iterate; return to step 1 if a long-term goal cannot be expressed.

## Long-term goals

- Support complex numbers
- Compute derivatives of functions
- Find roots of some polynomial equations
- Find some definite and indefinite integrals
- Solve some ordinary differential equations