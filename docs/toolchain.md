# Toolchain Guide

HMAT ships with a complete developer toolchain. Every tool is designed to work together,
and every tool follows the same principle: be helpful, not verbose.

---

## Overview

| Tool        | Command  | Purpose                              |
|-------------|----------|--------------------------------------|
| Compiler    | `hmatc`  | Compile `.hm` source to native binary|
| Package mgr | `hpkg`   | Manage dependencies and packages     |
| Formatter   | `hfmt`   | Auto-format source code              |
| Language server | `hmat-lsp` | IDE integration (LSP)           |
| REPL        | `hmat`   | Interactive shell                    |

> **Status:** Only `hmatc` is currently under active development (Phase 0–1).
> Other tools are planned for Phase 4.

---

## hmatc — The Compiler

`hmatc` compiles HMAT source files to native binaries via LLVM.

### Basic Usage

```bash
# Compile and link (produces ./hello)
hmatc hello.hm

# Specify output name
hmatc hello.hm -o greet

# Compile with optimizations
hmatc hello.hm --release

# Compile for WebAssembly
hmatc hello.hm --target=wasm -o hello.wasm
```

### Emit Flags

Inspect intermediate representations without producing a binary:

```bash
# Print the token stream (lexer output) — available now
hmatc --emit=tokens hello.hm

# Print the AST (parser output) — available now
hmatc --emit=ast hello.hm

# Print the HIR (desugared, type-annotated) — Phase 1
hmatc --emit=hir hello.hm

# Print LLVM IR — Phase 1
hmatc --emit=llvm-ir hello.hm
```

#### `--emit=ast` example

Given `hello_world.hm`:

```hmat
fn main():
    print("Hello, HMAT!")
```

`hmatc --emit=ast hello_world.hm` produces:

```
Program
  FunctionDecl `main`
    params: (none)
    return: (none)
    body:
      Block
        ExprStmt
          Call
            callee:
              Ident `print`
            args:
              StrLit "Hello, HMAT!"
```

Each line is two spaces deeper per nesting level. Spans are omitted — the tree is for reading, not round-tripping.

### Error Output

`hmatc` errors follow a consistent format:

```
error[E001]: unknown token `$` at line 3, column 7
  --> hello.hm:3:7
   |
 3 |     let x = $value
   |             ^
   |
help: this character is not part of HMAT syntax — remove it or check for a typo
```

Every error has:
- A code (`E001`–`E999`) for documentation lookup
- Exact source location with a caret
- A `help` line that says what to do

### AI Usage Report

```bash
# Analyze AI construct usage in your program
hmatc --analyze-ai hello.hm
```

Shows which models are declared, which methods are called, and estimated token usage.

---

## hpkg — Package Manager

> **Status:** Planned for Phase 4.

```bash
# Create a new project
hpkg init my-project
cd my-project

# Add a dependency
hpkg add hmat-http

# Remove a dependency
hpkg remove hmat-http

# Build the project
hpkg build

# Run the project
hpkg run

# Publish to the registry
hpkg publish
```

### hmat.toml

Projects are configured by `hmat.toml`:

```toml
[project]
name = "my-project"
version = "0.1.0"
author = "Your Name <you@example.com>"

[dependencies]
hmat-http = "1.2"
hmat-json = "0.8"

[dev-dependencies]
hmat-test = "0.4"
```

---

## hfmt — Formatter

> **Status:** Planned for Phase 4.

`hfmt` formats HMAT source code. It is opinionated and idempotent: running it twice
produces the same output.

```bash
# Format a file in-place
hfmt hello.hm

# Format all .hm files in the project
hfmt .

# Check formatting (exit 1 if any file would change)
hfmt --check .
```

### What hfmt Enforces

- 4-space indentation (tabs converted)
- No trailing whitespace
- Consistent blank lines between top-level items
- Consistent spacing around operators
- Consistent import ordering

---

## hmat-lsp — Language Server

> **Status:** Planned for Phase 4.

`hmat-lsp` implements the Language Server Protocol, enabling IDE features in any
LSP-compatible editor.

### Features

- **Hover** — show type, documentation, and definition
- **Go-to definition** — jump to where a symbol is defined
- **Completion** — context-aware suggestions including field names and method names
- **Inline errors** — compiler errors shown as you type
- **Rename** — rename a symbol across the project
- **Format on save** — integrates with `hfmt`

### VS Code Setup

Install the HMAT extension from the VS Code marketplace. It bundles `hmat-lsp`
and configures everything automatically.

Manual setup for other editors: point your LSP client at the `hmat-lsp` binary.

---

## hmat — The REPL

> **Status:** Planned for Phase 4.

An interactive shell for experimenting with HMAT:

```
$ hmat
HMAT v0.2.0 — type :help for commands
>>> let x = 42
>>> x * 2
84
>>> fn double(n: int) -> int:
...     n * 2
>>> double(x)
84
>>> ai model assistant = load("anthropic/claude-3-5-sonnet")
>>> await assistant.chat("Hello!")
"Hello! How can I help you today?"
```

Commands:
- `:help` — show available commands
- `:type x` — show the type of expression `x`
- `:doc fn_name` — show documentation for a function
- `:quit` — exit

---

## Building from Source

To build the entire toolchain from the monorepo:

```bash
git clone https://github.com/hassanzaibhayat/hmat
cd hmat
cargo build --release --workspace
```

Binaries land in `target/release/`:
- `hmatc` — compiler (available now)
- `hpkg`, `hfmt`, `hmat-lsp`, `hmat` — planned tools

---

## See Also

- [Getting Started](getting-started.md) — using hmatc for the first time
- [CHANGELOG](../CHANGELOG.md) — what's been built
- [Roadmap](../spec/roadmap.md) — when each tool ships
