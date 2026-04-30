# Syntax open questions

Checklist of syntax decisions still to make. Update by removing items as they are settled (and recording the decision in `syntax-notes.md`).

Grouped by how soon each one blocks writing realistic example files.

## Blocks coherent example files

(none — the syntax surface is settled enough to start writing example files)

## Important, can be sketched after first examples

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
