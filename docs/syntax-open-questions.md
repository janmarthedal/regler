# Syntax open questions

Checklist of syntax decisions still to make. Update by removing items as they are settled (and recording the decision in `syntax-notes.md`).

Grouped by how soon each one blocks writing realistic example files.

## Blocks coherent example files

(none — the syntax surface is settled enough to start writing example files)

## Important, can be sketched after first examples

- [ ] **Queries and rewriting syntax.** How a user actually does something — `simplify`, `rewrite using fact_name`, `evaluate`, `prove`. Decide together with the next two items, since they interact.
- [ ] **Naming facts.** Whether facts have names so they can be referred to later (e.g., when invoking a rewrite). `fact comm_add : ∀ x, y ∈ ℝ. x + y = y + x` vs. anonymous. Lands together with queries/rewriting since names matter only when facts are invoked.
- [ ] **Direction of rewriting.** When a fact `a = b` is used as a rewrite, how `→` vs. `←` is specified. Default? Both?
- [ ] **Pattern variables vs. bound variables.** Whether variables bound by `∀ x ∈ ℝ. ...` automatically become pattern variables matching arbitrary subterms when the fact is used as a rewrite.
- [ ] **Condition language inside `if`.** Currently conjunctions of membership/equality/inequality; whether `∨`, `¬`, quantifiers are allowed.
- [ ] **Auto-unfolding of definitions.** Whether `let half : ℚ = 1/2` causes `half` to be unfolded automatically or only when explicitly rewritten.
- [ ] **Set membership vs. promotion in expressions.** Whether `2 + π` (with `2 ∈ ℕ`, `π ∈ ℝ`) requires explicit coercion or is promoted implicitly. Big readability impact.
- [ ] **Overloaded operators under inference.** Tie-breaking rule when an operator like `·` has signatures on multiple sets and the inferred type of an expression (anywhere — not just lambda bodies) is ambiguous. Current proposal: smallest set with a defined signature.
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
