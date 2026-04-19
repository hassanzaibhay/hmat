# HMAT Language Specification v0.2
# Roadmap
# Author: Hassan Zaib Hayat <hassanzaibhayatske@gmail.com>

---

## Vision

HMAT will be the language that developers reach for when they want to build
AI-native applications with the performance of systems languages and the joy of scripting.

By v1.0, a developer should be able to:
- Write a production web server in HMAT
- Train, fine-tune, and call AI models without leaving the language
- Ship a binary that runs anywhere with no runtime dependency
- Onboard a new team member in one afternoon

---

## Phase 0 — Foundation
**Goal:** The compiler exists. It tokenizes and parses HMAT source.

| Task                              | Status  | Notes                              |
|-----------------------------------|---------|------------------------------------|
| Rust workspace (hmatc)            | 🔲      | `cargo new --workspace`            |
| Lexer — all tokens                | 🔲      | logos crate, indent/dedent         |
| Parser — core AST nodes           | 🔲      | hand-rolled recursive descent      |
| AST pretty-printer                | 🔲      | `--emit=ast` flag                  |
| Error infrastructure              | 🔲      | thiserror, miette for display      |
| CLI (hmatc)                       | 🔲      | clap, `--emit` flag                |
| Test suite (lexer + parser)       | 🔲      | > 50 tests                         |

**Milestone marker:** `hmatc --emit=ast examples/hello_world.hm` prints AST without crashing.

---

## Phase 1 — Hello World
**Goal:** A real HMAT program compiles and runs.

| Task                              | Status  | Notes                              |
|-----------------------------------|---------|------------------------------------|
| Type checker — primitives         | 🔲      | int, float, str, bool, ()          |
| Type checker — functions          | 🔲      | argument types, return types       |
| Type inference — basic            | 🔲      | let binding inference              |
| HIR (desugared AST)               | 🔲      | explicit types everywhere          |
| LLVM IR codegen — primitives      | 🔲      | inkwell                            |
| LLVM IR codegen — functions       | 🔲      | call conventions                   |
| print() built-in                  | 🔲      | links to C printf                  |
| main() entry point                | 🔲      | generates proper binary entry      |
| Friendly error messages           | 🔲      | all errors have help messages      |
| Integration test: hello world     | 🔲      | compiles and produces output       |

**Milestone marker:** `hmatc hello.hm && ./hello` prints "Hello, HMAT!"

---

## Phase 2 — Core Language
**Goal:** HMAT is a usable language for real programs.

| Task                              | Status  | Notes                              |
|-----------------------------------|---------|------------------------------------|
| Structs                           | 🔲      | fields, methods, impl blocks       |
| Enums + variants                  | 🔲      | data-carrying variants             |
| Pattern matching                  | 🔲      | match stmt, all patterns           |
| Generics                          | 🔲      | monomorphization                   |
| Traits                            | 🔲      | declaration + impl                 |
| Built-in traits (Clone, Debug...) | 🔲      | @derive attribute                  |
| Closures                          | 🔲      | capturing, fn types                |
| Result<T,E> + Option<T>           | 🔲      | built-in, ? operator               |
| Ownership checker (lite)          | 🔲      | move, borrow, lifetime inference   |
| String interpolation (f"...")     | 🔲      | compile-time formatting            |
| Collections (Vec, Map, Set)       | 🔲      | stdlib core collections            |
| For loops + iterators             | 🔲      | Iterator trait                     |
| Range expressions (..  ..=)       | 🔲      |                                    |
| List comprehensions               | 🔲      | [x for x in list if cond]         |
| Type aliases                      | 🔲      |                                    |
| Numeric casts (as keyword)        | 🔲      |                                    |
| Test framework (@test, @bench)    | 🔲      |                                    |

**Milestone marker:** All examples in `examples/` compile and run correctly.

---

## Phase 3 — AI Native
**Goal:** HMAT's killer feature ships.

| Task                              | Status  | Notes                              |
|-----------------------------------|---------|------------------------------------|
| `ai model` declaration            | 🔲      | AST node + semantic validation     |
| `await` keyword                   | 🔲      | async/await full support           |
| Async runtime                     | 🔲      | lightweight tokio-based runtime    |
| Model.chat()                      | 🔲      | HTTP call to provider API          |
| Model.summarize()                 | 🔲      |                                    |
| Model.extract::<T>()              | 🔲      | structured output parsing          |
| Model.classify()                  | 🔲      |                                    |
| Model.embed()                     | 🔲      |                                    |
| HmatCode type                     | 🔲      | typed AI-generated code            |
| unsafe ai / eval()                | 🔲      | sandboxed execution                |
| Pipeline declaration              | 🔲      | pipeline stages compose            |
| Pipeline.run()                    | 🔲      |                                    |
| AI security rules (compiler)      | 🔲      | no hardcoded keys, literal models  |
| `--analyze-ai` flag               | 🔲      | AI usage report                    |
| hmat::ai stdlib                   | 🔲      | ChatMessage, join, race            |
| Multi-provider support            | 🔲      | Anthropic, OpenAI, Ollama          |

**Milestone marker:** `examples/ai_chat.hm` compiles, runs, and returns a real AI response.

---

## Phase 4 — Toolchain
**Goal:** The developer experience is complete.

| Task                              | Status  | Notes                              |
|-----------------------------------|---------|------------------------------------|
| hpkg (package manager)           | 🔲      | init, add, remove, publish         |
| hpkg registry (hosted)           | 🔲      | pkg.hmat.dev                       |
| hfmt (formatter)                  | 🔲      | idempotent, opinionated            |
| hmat-lsp (language server)        | 🔲      | hover, go-to-def, completion       |
| VS Code extension                 | 🔲      | syntax, LSP integration            |
| WASM target                       | 🔲      | `hmatc --target=wasm`              |
| hmat REPL                         | 🔲      | interactive shell                  |
| cargo fuzz integration            | 🔲      | continuous fuzzing in CI           |
| Build system integration          | 🔲      | hmat.toml project file             |

---

## Phase 5 — Community
**Goal:** HMAT is ready for the world.

| Task                              | Status  | Notes                              |
|-----------------------------------|---------|------------------------------------|
| Full stdlib (io, net, math, etc.) | 🔲      |                                    |
| hmat.dev website                  | 🔲      | docs, playground, registry         |
| Interactive playground            | 🔲      | run HMAT in browser                |
| Getting Started guide             | 🔲      | 15-minute onboarding               |
| Language reference                | 🔲      | complete                           |
| Community Discord                 | 🔲      |                                    |
| Blog post: Why HMAT               | 🔲      | Hassan's essay                     |
| v1.0 release                      | 🔲      |                                    |

---

## Non-Goals (v1.0 scope)

These are explicitly out of scope for v1.0. Tracked for future versions.

- **Macro system** — planned for v1.1. Too complex to get right under time pressure.
- **Hot reload** — planned for v1.2.
- **Native GPU compute** — planned for v2.0 (AI training use case).
- **Package signing + reproducible builds** — planned for v1.1.
- **Multi-threading primitives** — async covers most use cases; threads planned for v1.1.
- **Unicode identifiers** — v0.3+ (ASCII only in v0.2).
- **Inline assembly** — v1.1+ (unsafe block sufficient for now).

---

## Design Decisions Log

Decisions that were debated and resolved. Documented so we don't revisit them.

### D001: Significant Indentation (Python-style)
**Decision:** Yes.  
**Rationale:** Eliminates braces debate, enforces readability, matches the "simple" goal.  
**Trade-off:** Makes copy-paste harder in some contexts. Accepted.

### D002: No Semicolons
**Decision:** Yes.  
**Rationale:** Newlines are sufficient statement separators. Less visual noise.  
**Trade-off:** Parser complexity for multi-line expressions. Solved by explicit line continuation with `\`.

### D003: Result/Option as Built-In Types, Not Just Library Types
**Decision:** Yes.  
**Rationale:** Error handling is core to the language, not an afterthought. Compiler can optimize.  
**Trade-off:** More compiler complexity. Worth it.

### D004: `await` Can Be Prefix or Postfix
**Decision:** Both allowed.  
**Rationale:** Prefix is clearer for simple calls. Postfix enables clean method chaining.  
**Trade-off:** Two ways to do the same thing. Acceptable — hfmt enforces project-level consistency.

### D005: Ergonomic Ownership (Not Full Rust)
**Decision:** Ownership without explicit lifetimes in 95% of cases.  
**Rationale:** Rust's ownership system is powerful but has a steep learning curve. HMAT aims to get the safety without the pain.  
**Trade-off:** Some unsafe programs that Rust would reject may compile in HMAT. Accepted — real safety for real programs is the goal, not theoretical purity.

### D006: AI Calls Are Always Async
**Decision:** Yes.  
**Rationale:** AI model calls are inherently IO-bound. Forcing async makes programs non-blocking by default.  
**Trade-off:** Functions calling AI must be async. This propagates. Acceptable — the async/await syntax is clean enough.

### D007: No Null — Only Option<T>
**Decision:** Yes.  
**Rationale:** Null pointer bugs are a solved problem. Option<T> is better.  
**Trade-off:** More verbose in some cases. The compiler helps — Option methods are ergonomic.

### D008: `^` for Power, Not XOR
**Decision:** `^` is power (`a^b = a to the power b`). `~~` is XOR (rare enough to accept unusual syntax).  
**Rationale:** `a^2` is universally understood as "a squared." Bitwise XOR is rare in most programs.  
**Trade-off:** Breaks convention from C-family languages. Intentional — HMAT optimizes for readability.
