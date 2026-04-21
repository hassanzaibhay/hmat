# Getting Started with HMAT

Welcome. HMAT is designed to feel familiar on day one and reward you for years.
If you've written Python, you'll feel at home immediately.
If you've written C++, you'll appreciate what we've removed.
If you're here for the AI-native features — you're in the right place.

---

## Install

> **Status:** Phase 1 complete — `hmatc` compiles `.hm` files to native binaries via C codegen + clang.
> **Prerequisite:** [LLVM/clang](https://releases.llvm.org/) must be on your PATH (or installed at the standard
> Windows path `C:\Program Files\LLVM\bin\clang.exe`).
> To build the compiler from source, see [Building from Source](#building-from-source) below.

### Building from Source

You'll need [Rust](https://rustup.rs) (stable, 1.75+) and [LLVM/clang](https://releases.llvm.org/) for the compile step.

```bash
git clone https://github.com/hassanzaibhayat/hmat
cd hmat
cargo build --release --workspace
# Compiler lands at: target/release/hmatc
# Add it to your PATH or invoke it with the full path.
```

---

## Your First Program

Create a file called `hello.hm`:

```hmat
fn main():
    print("Hello, HMAT!")
```

Compile and run:

```bash
# Linux / macOS
hmatc hello.hm
./hello

# Windows
hmatc hello.hm
.\hello.exe
```

Output:

```
Hello, HMAT!
```

That's it. No semicolons. No `public static void main`. No class wrapping a static method.
Just your intent, on the page.

> **How it works (Phase 1):** `hmatc` emits C11 source and invokes `clang -O2` to
> produce the binary. The intermediate `.c` file is deleted on success.
> Use `hmatc --emit=c hello.hm` to inspect the generated C.

---

## The Basics in Five Minutes

### Variables

```hmat
let name = "HMAT"        # type inferred as str
let version: int = 2     # explicit annotation — both are valid
let mut counter = 0      # mutable variables need `mut`
let pi = 3.14159         # float
let active = true        # bool
```

Types are inferred from the value on the right. You can always add an annotation — the compiler
will tell you if it disagrees.

### Functions

```hmat
fn greet(name: str) -> str:
    return f"Hello, {name}!"

fn add(a: int, b: int) -> int:
    a + b    # last expression is the implicit return
```

Parameters are typed. Return types follow `->`. If a function returns nothing, omit the `->`.

### Conditionals

```hmat
fn classify(n: int) -> str:
    if n > 0:
        return "positive"
    elif n < 0:
        return "negative"
    else:
        return "zero"
```

### Loops

```hmat
for i in 0..10:
    print(i)

for item in my_list:
    print(item)

let mut n = 0
while n < 5:
    n += 1
```

### Pattern Matching

```hmat
let score = 87

let grade = match score:
    90..=100 -> "A"
    80..=89  -> "B"
    70..=79  -> "C"
    _        -> "F"

print(f"Grade: {grade}")
```

`match` exhaustively covers all cases. If you miss one, the compiler tells you.

---

## Error Handling

HMAT has no exceptions. Errors are values — functions that can fail return `Result<T, E>`.

```hmat
fn divide(a: float, b: float) -> Result<float, str>:
    if b == 0.0:
        return Err("division by zero")
    return Ok(a / b)

match divide(10.0, 2.0):
    Ok(result) -> print(f"Answer: {result}")
    Err(msg)   -> print(f"Error: {msg}")
```

The `?` operator propagates errors automatically — no try/catch noise:

```hmat
fn compute(a: float, b: float) -> Result<float, str>:
    let x = divide(a, b)?     # returns early if Err
    return Ok(x * 2.0)
```

See the [Error Handling Guide](error-handling.md) for the full picture.

---

## Structs

```hmat
struct Point:
    x: float
    y: float

    fn new(x: float, y: float) -> Point:
        Point { x, y }

    fn display(self) -> str:
        return f"Point({self.x}, {self.y})"

let p = Point::new(3.0, 4.0)
print(p.display())    # Point(3.0, 4.0)
```

---

## AI-Native Features

HMAT's defining feature: AI models are first-class language constructs.

```hmat
ai model assistant = load("anthropic/claude-3-5-sonnet")

async fn main():
    let reply = await assistant.chat("What is HMAT?")
    print(reply)
```

No HTTP boilerplate. No JSON parsing. No provider SDK to install.
The language knows what an AI model is.

See the [AI Guide](ai-guide.md) for the full feature set.

---

## What's Next

You've seen the core. Where you go next depends on what you're building:

| I want to...                         | Read this                                    |
|--------------------------------------|----------------------------------------------|
| Learn the full syntax                | [Syntax Guide](syntax-guide.md)              |
| Understand types and type errors     | [Type System Guide](type-system.md)          |
| Understand memory / ownership        | [Ownership Guide](ownership-guide.md)        |
| Build AI-powered programs            | [AI Guide](ai-guide.md)                      |
| Handle errors properly               | [Error Handling Guide](error-handling.md)    |
| Write async / concurrent code        | [Async Guide](async-guide.md)                |
| Use the compiler and tools           | [Toolchain Guide](toolchain.md)              |
| Read working programs                | [Examples](../examples/README.md)            |

---

## See Also

- [Syntax Guide](syntax-guide.md) — complete language reference
- [Type System Guide](type-system.md) — types, inference, error codes E100–E110
- [Examples](../examples/README.md) — learn by reading real programs
- [CHANGELOG](../CHANGELOG.md) — what's been built, what's coming
