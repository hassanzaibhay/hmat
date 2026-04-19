# CHANGELOG

All notable changes to the HMAT programming language are documented here.

Format: `[Unreleased]` for work in progress, `[vX.Y.Z - YYYY-MM-DD]` for releases.  
This file is maintained by the **Docs Agent** and updated after every session.

---

## [Unreleased]

### Status
Phase 0 — Foundation. Lexer landed.

### Fixed
- Lexer tests: 19 failures in `hmatc/tests/lexer_tests.rs` caused by test
  expectations omitting the trailing `Newline` token the lexer correctly emits
  for single-line inputs. Token-list assertions gained a final `Token::Newline`;
  count-based assertions (`float_literals`, `float_with_exponent`,
  `string_escapes_are_decoded`, `f_string_preserves_interior_for_parser`) were
  incremented by one. Lexer code itself was not changed. `cargo test
  --workspace` now reports 37 passed, 0 failed.

### Added
- `hmatc` Cargo workspace (`Cargo.toml`) with the `hmatc` binary+library crate
- Lexer: complete tokenization of all Phase 0 HMAT tokens per spec v0.2 §2
  (keywords, identifiers, int/float/hex/bin/oct literals with `_` separators,
  string literals with escape processing including `\xHH`, f-string bodies,
  all arithmetic/comparison/assignment/arrow/borrow/bitwise-unary operators,
  all delimiters including `..`, `..=`, `::`)
- Lexer: significant-whitespace handling — `Indent`/`Dedent` injection over
  a 4-column indent model (tabs count as 4), blank and comment-only lines do
  not affect indentation, trailing dedents emitted at EOF
- Lexer: friendly `LexError` with codes (`E001` unknown token, `E002`
  malformed literal, `E003` indent mismatch), span tracking, and help text
- CLI: `hmatc --emit=tokens` prints the token stream in a greppable format;
  `--emit=ast|hir|llvm-ir` stubbed with "not implemented yet" messages
- Stubs for `parser` and `ast` modules so the workspace compiles end-to-end
- Shared `CompilerError` enum (`error.rs`) wrapping IO + lex errors
- `hmatc/tests/lexer_tests.rs` — 30+ integration tests covering every token
  category, indent edge cases, span correctness, and end-to-end hello-world
  tokenization

### Completed (pre-session)
- Spec v0.2: language grammar, type system, ownership model, AI constructs
- Initial documentation scaffolding
- Project architecture defined
- Agent system designed (7 agents: dev, test, debug, qa, security, docs, clean)
- CLAUDE.md: full project brain
- INSTRUCTIONS.md: development workflow
- FIRST_PROMPT.md: ignition for Phase 0

### Known Issues
- (resolved) `cargo test --workspace` — previously unverified on the author's
  machine due to missing Windows SDK libs. Now runs green: 37 passed, 0 failed.

### Next
- Parser: AST for core syntax
- Basic type checker
- Hello World end-to-end compilation

---

## Roadmap Milestones

| Milestone            | Description                          | Status      |
|----------------------|--------------------------------------|-------------|
| v0.1.0 — Foundation  | Lexer + Parser + basic type checker  | 🔲 Pending  |
| v0.2.0 — Hello World | Compiles and runs first program      | 🔲 Pending  |
| v0.3.0 — Core        | Functions, structs, enums, generics  | 🔲 Pending  |
| v0.4.0 — AI Native   | ai model, await, pipeline            | 🔲 Pending  |
| v0.5.0 — Toolchain   | hpkg, hfmt, hmat-lsp                 | 🔲 Pending  |
| v1.0.0 — Community   | Stdlib, registry, launch             | 🔲 Pending  |
