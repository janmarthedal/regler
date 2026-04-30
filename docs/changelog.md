# Changelog

Per-version log of program changes. Versions match `Cargo.toml`.

## 0.2.0

Milestone 3: kernel term representation and `evaluate`.

- New `kernel` module with a uniform-prefix `Term` ADT (`Nat(BigUint)`, `Var(Symbol)`, `App(Symbol, Vec<Term>)`). Symbols are `Rc<str>`; an interner is deferred until rewriting hot paths exist.
- Lowering from surface AST to kernel terms. Integer literals lower to `Nat`; binary operators lower to `App` with the operator symbol as head. Negative literals are rejected (no `Int` until ℤ arrives in milestone 6).
- Capture-free substitution `subst(&Term, &HashMap<Symbol, Term>)` over the kernel. No binders yet, so substitution is a plain tree walk.
- Bottom-up `evaluate` performing literal arithmetic on ℕ for `+`, `·`, and `^`. Closed integer subterms reduce; symbolic subterms are preserved (e.g. `x + 2 · 3` becomes `x + 6`). Exponents must fit in `u32`.
- Kernel-to-surface printing reuses the existing pretty-printer by lowering kernel terms back into `ast::Expr`.
- New `evaluate <expr>` REPL command. `let` bindings are substituted into the expression before evaluation, so `let x = 7` followed by `evaluate x · x + 1` prints `50`.
- Added `num-traits` dependency for `ToPrimitive` on `BigUint`.
- Integration tests for arithmetic, big-integer cases, partial symbolic reduction, evaluator identity on purely symbolic inputs, and substitution-resolved bindings.

## 0.1.0

Milestone 2: round-trip REPL on a minimal surface.

- Cargo crate scaffolded with `num-bigint` for arbitrary-precision integer literals.
- Lexer for identifiers, integer literals, `+`, `·`, `^`, `=`, `(`, `)`, and the keywords `let`, `fact`, `print`.
- AST with a single `BinOp` variant; `=` is a regular binary operator at the lowest precedence. Precedence order: `= < + < · < ^`. `^` is right-associative; `+` and `·` are left-associative; `=` is treated as non-associative by the printer.
- Pratt-style precedence-climbing parser. `fact` accepts any expression (kernel will validate fact shape later).
- Pretty-printer that parenthesizes only when needed to preserve `parse(print(t)) == t`.
- Line-based REPL accepting `let name = expr`, `fact <expr>`, and `print <expr>`. `print` resolves a bare identifier through `let` bindings (one level); everything else prints verbatim. Facts are stored but not yet used.
- Round-trip integration tests covering atoms, flat binops, precedence mixes, associativity, equality at top level, and commands.
