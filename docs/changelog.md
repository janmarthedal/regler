# Changelog

Per-version log of program changes. Versions match `Cargo.toml`.

## 0.1.0

Milestone 2: round-trip REPL on a minimal surface.

- Cargo crate scaffolded with `num-bigint` for arbitrary-precision integer literals.
- Lexer for identifiers, integer literals, `+`, `·`, `^`, `=`, `(`, `)`, and the keywords `let`, `fact`, `print`.
- AST with a single `BinOp` variant; `=` is a regular binary operator at the lowest precedence. Precedence order: `= < + < · < ^`. `^` is right-associative; `+` and `·` are left-associative; `=` is treated as non-associative by the printer.
- Pratt-style precedence-climbing parser. `fact` accepts any expression (kernel will validate fact shape later).
- Pretty-printer that parenthesizes only when needed to preserve `parse(print(t)) == t`.
- Line-based REPL accepting `let name = expr`, `fact <expr>`, and `print <expr>`. `print` resolves a bare identifier through `let` bindings (one level); everything else prints verbatim. Facts are stored but not yet used.
- Round-trip integration tests covering atoms, flat binops, precedence mixes, associativity, equality at top level, and commands.
