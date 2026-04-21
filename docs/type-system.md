# HMAT Type System

HMAT is statically typed with full inference. Type annotations are optional — write them for clarity, not for the compiler. If it compiles, it is type-safe.

No null. No unchecked errors. No implicit coercion.

---

## The Simple Case

```hmat
fn add(a: int, b: int) -> int:
    a + b

fn main():
    let result = add(1, 2)   # type of result inferred as int
    print(result)
```

The compiler infers `result` as `int` from the return type of `add`. No annotation needed.

---

## Primitive Types

These are the types the compiler understands today (Phase 0). More types — shapes, sum types, generics, collections — are added in later phases.

| Type    | Description                          | Literal examples        |
|---------|--------------------------------------|-------------------------|
| `int`   | 64-bit signed integer (default)      | `0`, `42`, `-7`, `1_000_000` |
| `float` | 64-bit IEEE-754 float (default)      | `3.14`, `-0.5`, `1.0e9` |
| `str`   | UTF-8 string                         | `"hello"`, `f"hi {name}"` |
| `bool`  | Boolean                              | `true`, `false`         |
| `()`    | Unit — no meaningful value           | implicit on void returns |

Sized integer aliases (`i8`, `i16`, `i32`, `i64`, `i128`, `u8`–`u128`) are accepted and treated as `int` in Phase 0. Float aliases `f32` and `f64` are treated as `float`.

---

## Type Inference

The compiler infers types from context. You rarely need to annotate.

```hmat
fn main():
    let name    = "HMAT"     # str
    let count   = 0          # int
    let ratio   = 0.5        # float
    let enabled = true       # bool
```

When inference is ambiguous — specifically when a `let` binding is initialised with `nil` — you must supply an annotation:

```hmat
fn main():
    let x: int = nil    # ok — annotation resolves nil
    let y      = nil    # E101: cannot infer type for `y`
```

---

## Type Annotations

Annotations go on `let` bindings and function signatures. They are always optional on `let`; they are required on function parameters.

```hmat
# Annotated let — compiler verifies the annotation matches the value
let x: int   = 42
let pi: float = 3.14159
let msg: str  = "hello"

# Unannotated let — type inferred from value
let y = 42      # int
let z = 3.14    # float
```

If an annotation is present and does not match the value, the compiler emits `E100`.

---

## Function Types

Every function has a signature. Parameters must be annotated. The return type is optional and defaults to `()` (unit) when omitted.

```hmat
# No return type — returns ()
greet(name: str):
    print(f"Hello, {name}!")

# Explicit return type
add(a: int, b: int) -> int:
    a + b

# Return type can be inferred from annotation on the return statement,
# but an explicit annotation is preferred for readability
```

The compiler checks:
- Every argument's type matches the corresponding parameter (E107)
- The number of arguments matches the parameter count (E106)
- Every `return` value's type matches the declared return type (E109)

---

## Operators and Types

Operators are not polymorphic in Phase 0 — each is defined on specific types.

### Arithmetic (`+`, `-`, `*`, `/`, `%`, `^`)

Defined for `int` and `float`. Both operands must be the same numeric type. Mixed arithmetic (`int + float`) is not allowed — convert explicitly.

```hmat
fn main():
    let a = 10 + 3      # int
    let b = 1.5 * 2.0   # float
    let c = 10 + 1.5    # E102: `+` is not defined for `int` and `float`
```

### Comparison (`==`, `!=`, `<`, `<=`, `>`, `>=`)

Defined for `int`, `float`, and `str`. Produces `bool`.

```hmat
let ordered = 3 < 5     # bool: true
let same    = "a" == "a" # bool: true
```

### Logical (`and`, `or`)

Defined for `bool` operands only. Produces `bool`.

```hmat
let both = true and false   # bool: false
let either = false or true  # bool: true
let bad = 1 and true        # E102: `and` not defined for `int` and `bool`
```

### Unary

| Operator | Types      | Result  |
|----------|------------|---------|
| `-`      | `int`, `float` | same type |
| `not`    | `bool`     | `bool`  |

---

## Type Errors (E100–E110)

Every type error has a code, a location, and a help message — errors are teachers, not walls.

### E100 — Type mismatch

A value's type does not match its annotation.

```hmat
let x: int = "hello"   # E100: expected `int`, found `str`
```

**Help:** Remove the annotation and let the type be inferred, or change the value to match the annotation.

---

### E101 — Cannot infer type

A binding is initialised with `nil` and has no annotation to resolve it.

```hmat
let x = nil   # E101: cannot infer type for `x`
```

**Help:** Add a type annotation: `let x: int = nil`

---

### E102 — Undefined binary operator

An operator is applied to types it is not defined for.

```hmat
let x = true + false   # E102: `+` is not defined for `bool` and `bool`
```

**Help:** Check the operand types. Arithmetic requires `int` or `float`; `and`/`or` requires `bool`.

---

### E103 — Undefined unary operator

A unary operator is applied to a type it is not defined for.

```hmat
let x = -true   # E103: `-` is not defined for `bool`
```

**Help:** `-` applies to `int` and `float`. `not` applies to `bool`.

---

### E104 — Undefined variable

An identifier is used before it is declared.

```hmat
fn main():
    print(x)   # E104: undefined variable `x`
```

**Help:** Declare `x` with a `let` binding before using it.

---

### E105 — Undefined function

A call to a name that is not declared as a function.

```hmat
fn main():
    compute()   # E105: undefined function `compute`
```

**Help:** Check the spelling and make sure the function is declared at the top level.

---

### E106 — Wrong argument count

A function is called with the wrong number of arguments.

```hmat
add(a: int, b: int) -> int: a + b

fn main():
    add(1)       # E106: `add` expects 2 argument(s), got 1
    add(1, 2, 3) # E106: `add` expects 2 argument(s), got 3
```

**Help:** Check the function signature and pass the correct number of arguments.

---

### E107 — Argument type mismatch

An argument's type does not match the parameter's declared type.

```hmat
greet(name: str): print(name)

fn main():
    greet(42)   # E107: argument 1 of `greet`: expected `str`, found `int`
```

**Help:** Convert the value to the expected type before passing it.

---

### E108 — Not callable

An attempt to call a non-function value.

```hmat
fn main():
    let x = 42
    x()   # E108: `int` is not callable
```

**Help:** Only functions can be called. Check that you're using the correct name.

---

### E109 — Return type mismatch

A `return` value's type differs from the function's declared return type.

```hmat
greet() -> str:
    return 42   # E109: return type mismatch in `greet` — expected `str`, found `int`
```

**Help:** Either change the return type annotation or change the returned value.

---

### E110 — Unknown type

A type annotation references a name the compiler does not recognise.

```hmat
let x: Colour = "red"   # E110: unknown type `Colour`
```

**Help:** Check the spelling. Phase 0 supports: `int`, `float`, `str`, `bool`, `()`.
User-defined shapes and sum types are not yet available (Phase 2).

---

## How the Checker Works

The type checker runs as a single pass over the AST after parsing. It:

1. Registers all top-level function signatures in a first pass (enabling forward calls).
2. Walks each function body in a second pass, checking statements and expressions.
3. Collects all errors rather than stopping at the first — one compilation run surfaces as many problems as possible.
4. Uses an `Unknown` sentinel type to suppress cascading errors: once an error fires, downstream expressions that depend on the bad value do not produce additional noise.

---

## See Also

- [Syntax Guide](syntax-guide.md) — full language syntax
- [Error Handling](error-handling.md) — `Result`, `Option`, fallible functions
- [Ownership Guide](ownership-guide.md) — memory model (Phase 2)
- [spec/0.2/types.md](../spec/0.2/types.md) — formal type system specification
