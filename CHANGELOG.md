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

### Added (2026-04-21 — Type Checker session)
- `hmatc/src/semantic/types.rs` — Phase 0 type checker (~450 lines):
  - `Type` enum: `Int`, `Float`, `Str`, `Bool`, `Unit`, `Nil`, `Unknown`
  - `check(&Program)` public entry point — one-pass tree walk, collects all
    errors rather than stopping at the first
  - Two-pass function resolution: pass 1 registers all signatures (enables
    forward calls), pass 2 checks bodies
  - Primitive type inference for all literal kinds (int, float, str, bool, nil)
  - `let` binding checking: infers type from RHS; validates annotation if present;
    errors when `nil` has no annotation (E101)
  - Function argument checking: arity (E106), per-argument type (E107)
  - `return` statement checking against declared return type (E109)
  - Binary operator type checking — arithmetic on numeric types, comparison
    producing `bool`, logical `and`/`or` on booleans (E102)
  - Unary operator checking — `-` on numeric, `not` on bool (E103)
  - Identifier resolution with scope stack (E104)
  - Function call resolution (E105)
  - Type annotation parsing — all sized integer aliases (`i8`–`i128`,
    `u8`–`u128`), float aliases (`f32`, `f64`), unknown type names (E110)
  - `Unknown` sentinel suppresses cascading errors after the root cause fires
  - Phase 0 builtin: `print(str) -> ()` wired in without a stdlib loader
- `hmatc/src/semantic/mod.rs` — `pub use types::{check, TypeError, Type}`
- `hmatc/src/error.rs` — `CompilerError::Type` variant; `From<Vec<TypeError>>`
- `hmatc/src/main.rs` — `--emit=ast` pipeline now runs the type checker after
  parsing; each `TypeError` printed with its code and `help` line
- `hmatc/tests/type_checker_tests.rs` — 38 integration tests:
  happy-path coverage for all primitive types, let bindings, function calls,
  return types, operators; error-path coverage for every E-code E100–E110;
  milestone test `cli_emit_ast_hello_world` verifies the full pipeline
- `docs/type-system.md` — new: phase-0 type system guide (primitives,
  inference, annotations, operators, error reference)
- `CHANGELOG.md` — this entry

### Verified (2026-04-21)
- `cargo test --workspace` — **171 passed, 0 failed, 3 ignored**
  (68 lexer + 63 parser + 38 type checker + 2 doc tests)
- `cargo clippy --workspace --all-targets -- -D warnings` — zero warnings
- `cargo fmt --check` — clean
- Milestone: `hmatc --emit=ast examples/hello_world.hm` type-checks with
  zero errors

### Audited (2026-04-21 — Foundation audit session)
- Foundation audit complete — lexer, parser, AST vs spec v0.3.
- See `audit/FOUNDATION-AUDIT-2026.md` for full findings.
- Three HIGH RISK spec-drift items found and fixed in the same session:
  - **Lexer keywords** aligned to spec v0.3: added `shape` / `on` / `flow` /
    `fail`; removed obsolete `match` / `struct` / `enum` / `trait` / `impl` /
    `load` / `pipeline` / `where` / `?` variants.
  - **Parser generics** now use `[T]` (spec v0.3 §2.7 / §3.1) instead of
    `<T>`. `<` and `>` remain comparison operators only.
  - **Return types** now accept fallible/optional suffixes per spec v0.3
    §3.1 / §4: `-> T or Fail`, `-> T or nil`, `-> T or Fail or nil`.
    `TypeRef` gained `is_fallible` / `is_nilable` flags; new
    `parse_return_type()` helper restricts these suffixes to return position.
- Test updates: lexer keyword cases refreshed; 4 new parser tests
  (`function_with_fallible_return_type`, `function_with_nilable_return_type`,
  `function_with_fallible_and_nilable_return_type`,
  `generic_return_type_with_single_param`); existing
  `function_with_generic_return_type` switched from `<...>` to `[...]`;
  `ai_model_decl_tokenizes` updated to spec-v0.3 form.

### Verified (2026-04-21 — post-audit)
- `cargo test --workspace` — **177 passed, 0 failed, 3 ignored**
  (69 lexer + 67 parser + 38 type checker + 3 doc tests)
- `cargo clippy --workspace --all-targets -- -D warnings` — clean
- Milestone: `hmatc --emit=ast examples/hello_world.hm` — still end-to-end green

### Next
- LLVM IR codegen bootstrap (Phase 1 milestone)
- Hello World end-to-end: source → binary → executable

---

## Roadmap Milestones

| Milestone            | Description                          | Status      |
|----------------------|--------------------------------------|-------------|
| v0.1.0 — Foundation  | Lexer + Parser + basic type checker  | ✅ Done     |
| v0.2.0 — Hello World | Compiles and runs first program      | 🔲 Pending  |
| v0.3.0 — Core        | Functions, structs, enums, generics  | 🔲 Pending  |
| v0.4.0 — AI Native   | ai model, await, pipeline            | 🔲 Pending  |
| v0.5.0 — Toolchain   | hpkg, hfmt, hmat-lsp                 | 🔲 Pending  |
| v1.0.0 — Community   | Stdlib, registry, launch             | 🔲 Pending  |
