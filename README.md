# HMAT

> AI-native. Simple like Python. Powerful like C++. Optimized like Rust. Scalable like Java.

HMAT is a general-purpose programming language built for the AI era.

```hmat
ai model assistant = load("anthropic/claude-3-5-sonnet")

async fn main():
    let reply = await assistant.chat("What can HMAT do?")
    print(reply)
```

---

## Features

- **AI-native** — `ai model` is a first-class language construct, not a library
- **Simple syntax** — readable without a language reference; no semicolons, no boilerplate
- **Memory safe** — ownership model gives you safety without the Rust learning cliff
- **Fast** — LLVM backend, zero-cost abstractions, competes with C++ in hot paths
- **Friendly errors** — every error names the problem and suggests the fix
- **Statically typed** — full type inference; annotations optional, never forced

---

## Quick Look

```hmat
# Variables — types inferred, annotations optional
let name = "HMAT"
let mut counter = 0

# Functions are clean
fn greet(name: str) -> str:
    return f"Hello, {name}!"

# Errors are values — no exceptions
fn divide(a: float, b: float) -> Result<float, str>:
    if b == 0.0:
        return Err("division by zero")
    return Ok(a / b)

# Pattern matching is exhaustive and expressive
match divide(10.0, 0.0):
    Ok(result) -> print(f"Answer: {result}")
    Err(msg)   -> print(f"Error: {msg}")

# AI is just part of the language
ai model assistant = load("anthropic/claude-3-5-sonnet")

async fn summarize(text: str) -> Result<str, AiError>:
    return Ok(await assistant.summarize(text)?)
```

---

## Status

HMAT is under active development.

| Component          | Status              |
|--------------------|---------------------|
| Spec v0.2          | ✅ Complete          |
| Compiler (`hmatc`) | 🔨 Phase 0 in progress — Lexer ✅ Parser ✅, type checker next |
| Package manager    | 🔲 Phase 4           |
| Formatter          | 🔲 Phase 4           |
| Language server    | 🔲 Phase 4           |
| Standard library   | 🔲 Phase 5           |

See [CHANGELOG.md](CHANGELOG.md) for the detailed build log and [spec/roadmap.md](spec/roadmap.md)
for the full plan.

---

## Build from Source

You'll need [Rust](https://rustup.rs) (stable, 1.75+).

```bash
git clone https://github.com/hassanzaibhayat/hmat
cd hmat
cargo build --release --workspace
# Compiler: target/release/hmatc
```

Run the test suite:

```bash
cargo test --workspace
```

---

## Documentation

| Guide                                        | What It Covers                    |
|----------------------------------------------|-----------------------------------|
| [Getting Started](docs/getting-started.md)   | First steps, basics in 5 minutes |
| [Syntax Guide](docs/syntax-guide.md)         | Complete language reference       |
| [Ownership Guide](docs/ownership-guide.md)   | Memory model, borrowing           |
| [Error Handling](docs/error-handling.md)     | Result, Option, ? operator        |
| [AI Guide](docs/ai-guide.md)                 | AI-native features                |
| [Async Guide](docs/async-guide.md)           | Async/await, concurrency          |
| [Toolchain](docs/toolchain.md)               | hmatc, hpkg, hfmt, hmat-lsp      |
| [Examples](examples/README.md)               | Working programs to read          |

---

## Examples

```bash
# Hello, World
hmatc examples/hello_world.hm && ./hello_world

# Core language features
hmatc examples/core_language.hm && ./core_language

# AI chat (requires ANTHROPIC_API_KEY)
hmatc examples/ai_chat.hm && ./ai_chat
```

See [examples/README.md](examples/README.md) for the full list.

---

## Design Decisions

Notable choices, each made deliberately:

- **Significant indentation** — eliminates the braces debate, enforces readable code
- **No null** — only `Option<T>`. The compiler ensures you handle the absent case.
- **`^` is power, not XOR** — `a^2` means a squared; bitwise XOR uses `~~`
- **Ergonomic ownership** — ownership safety without explicit lifetime annotations in 95% of cases
- **AI calls are always async** — AI is IO; forcing async makes programs non-blocking by default

See [spec/roadmap.md](spec/roadmap.md) for the full design decisions log.

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for how to contribute.

The project uses a 7-agent development system (Dev, Test, Debug, QA, Security, Docs, Clean).
If you're contributing code, read [INSTRUCTIONS.md](INSTRUCTIONS.md) first.

---

## Author

Hassan Zaib Hayat — built with love for the developer community.

The goal: a language that developers will still be grateful for in 2035.
