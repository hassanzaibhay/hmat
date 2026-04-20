# CHANGELOG

All notable changes to the HMAT programming language are documented here.

Format: `[Unreleased]` for work in progress, `[vX.Y.Z - YYYY-MM-DD]` for releases.  
This file is maintained by the **Docs Agent** and updated after every session.

---

## [Unreleased]

### Status
Phase 0 — Foundation. Lexer landed. Parser landed.

### Added (2026-04-19 — Parser session)
- `hmatc/src/ast/mod.rs` — AST node types for Phase 0: `Program`, `Item`,
  `FunctionDecl`, `Param`, `TypeRef`, `Block`, `Statement`, `LetStmt`,
  `ReturnStmt`, `Identifier`, `Expression` (Literal / Identifier / Call /
  FieldAccess / Binary / Unary / Grouped), `Literal` (Int / Float / Str /
  FStr / Bool / Nil), `BinOp`, `UnOp`. Every node carries a byte span.
- `hmatc/src/ast/mod.rs` — indent-based AST pretty printer
  (`Program::pretty_print()`), consumed by the `--emit=ast` CLI.
- `hmatc/src/parser/mod.rs` — hand-rolled recursive descent parser over
  `Vec<(Token, Span)>`. Recognises: top-level `fn` declarations, optional
  return types (named / generic / `&T` / `&mut T`), parameters (`[mut] name: ty`),
  `let` / `return` / expression statements inside `INDENT ... DEDENT` blocks,
  full precedence climb from `or` → `and` → `==`/`!=` → `<`/`<=`/`>`/`>=` →
  `+`/`-` → `*`/`/`/`%` → unary `-`/`not` → right-assoc `^` → postfix
  (`.field`, `(args)`) → primary (literals, identifiers, `( expr )`).
  Newlines are skipped inside `(...)` so multi-line arg/param lists parse.
- `ParseError` with codes `E010`–`E015`: `UnexpectedToken`, `UnexpectedEof`,
  `ExpectedIdentifier`, `ExpectedType`, `EmptyBlock`, `InvalidExpression`.
  Every variant has a `help()` message — Zen #2: errors are teachers.
- `CompilerError::Parse` variant wires `ParseError` into the driver error
  path.
- `hmatc --emit=ast <file>` implemented — runs lexer + parser and prints
  an indented AST tree.
- `hmatc/tests/parser_tests.rs` — 38 integration tests covering: the
  `hello_world.hm` milestone (parse + pretty-print), function
  declarations (no-params / typed / mut params / generic return / `&T` /
  `&mut T` / multiple top-level fns), let & return statement shapes,
  literal / identifier / call / field-access / method-chain / unary /
  grouped expression parses, precedence (`*` tighter than `+`, comparison
  looser than additive, `and` tighter than `or`, right-assoc `^`), and
  six error cases each asserting a specific `E0xx` code plus non-empty
  `help()` text.

### Verified (2026-04-19)
- `cargo build` — clean.
- `cargo test --workspace` — **108 passed, 0 failed** (68 lexer + 38
  parser + 2 doc tests).
- `cargo clippy --workspace --all-targets -- -D warnings` — clean.
  Pre-existing `approx_constant` lint on the `3.14` float-lexer literal
  silenced with a local `#[allow(clippy::approx_constant)]` and a
  comment explaining the literal is under test, not approximating π.
- Milestone: `hmatc --emit=ast examples/hello_world.hm` prints the AST
  without crashing.

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

### Documentation (this session)
- `docs/getting-started.md` — installation, 5-minute tour, cross-reference table
- `docs/syntax-guide.md` — complete language reference (variables, functions, structs,
  enums, traits, generics, closures, match, loops, ownership syntax, imports, unsafe)
- `docs/ownership-guide.md` — ownership model, shared/mutable borrows, lifetime inference,
  regions, clone, common errors and what they mean
- `docs/error-handling.md` — Result/Option, `?` operator, custom error types,
  `.map_err()`, when to use panic
- `docs/ai-guide.md` — model declarations, chat/summarize/classify/embed/extract,
  pipelines, parallel calls, security rules, HmatCode type
- `docs/async-guide.md` — async functions, join/race/spawn, timeouts, patterns
- `docs/toolchain.md` — hmatc flags, hpkg, hfmt, hmat-lsp, REPL (current + planned)
- `examples/README.md` — index of all examples with phase and status
- `README.md` — rewritten as a full public project homepage

### Next
- Basic type checker (semantic analysis pass)
- Hello World end-to-end compilation (LLVM codegen bootstrap)

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
