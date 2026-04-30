# Syntax open questions

Checklist of syntax decisions still to make. Update by removing items as they are settled (and recording the decision in `syntax-notes.md`).

Grouped by how soon each one blocks writing realistic example files.

## Blocks coherent example files

(none — the syntax surface is settled enough to start writing example files)

## Important, can be sketched after first examples

- [ ] **Queries and rewriting syntax.** How a user actually does something — `simplify`, `rewrite using fact_name`, `evaluate`, `prove`. Decide together with the next two items, since they interact.
- [ ] **Direction of rewriting (manual invocation).** Auto-orientation by term order handles equalities whose sides are strictly comparable. For incomparable equalities (factor/expand pairs, etc.), how the user specifies `→` vs. `←` when invoking a fact manually.
- [ ] **Term order choice.** Which well-founded term order the kernel uses for auto-orientation (lex, KBO, LPO, …) and how the per-symbol precedence is fixed.
- [ ] **AC recognition details.** Which fact patterns trigger AC marking, whether partial AC (commutative-only or associative-only) is handled, and how marking interacts with overloaded operators across subset chains.
- [ ] **Condition language inside `if`.** Currently conjunctions of membership/equality/inequality; whether `∨`, `¬`, quantifiers are allowed.
- [ ] **Auto-unfolding of definitions.** Whether `let half : ℚ = 1/2` causes `half` to be unfolded automatically or only when explicitly rewritten.
- [ ] **Narrowing proof obligations.** When a value is declared in a strict subset (e.g., `let small : Pos = 1/2`), how the kernel checks the membership obligation.

## Deferrable

- [ ] **File structure / modules / imports.** Single-file examples suffice to start.
- [ ] **Theorem keyword and proof syntax.** Not needed until proofs are written.
- [ ] **ASCII fallbacks.** Whether `in`, `subset`, `forall`, etc. are accepted alongside Unicode.
- [ ] **Sort hierarchy beyond `Set`.** Whether a higher universe is ever needed.
- [ ] **String literals, printing, I/O.** Only needed for runnable examples.
- [ ] **Pretty-printing rules.**

## Cross-cutting

- [ ] **The declaration-then-fact pattern's verbosity.** Verbose for long subset chains and parameterized sets. Acceptable for now; revisit if it becomes painful.
- [ ] **Whether to reintroduce sugar.** Function-definition sugar (form-4) and parameterized-set definition sugar were rejected. Keep an eye out for cases where their absence hurts readability badly.
