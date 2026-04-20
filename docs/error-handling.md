# Error Handling Guide

HMAT has no exceptions. Errors are values. If a function can fail, its return type says so.
This is not a limitation — it's a design decision that makes programs easier to reason about,
easier to test, and impossible to have silent failures.

---

## The Two Error Types

HMAT has two built-in types for fallibility:

- **`Result<T, E>`** — an operation that can fail with an error of type `E`, or succeed with a value of type `T`.
- **`Option<T>`** — a value that may or may not exist.

Both are enums built into the language. You use them with `match` or helper methods.

---

## Result\<T, E\>

```hmat
fn divide(a: float, b: float) -> Result<float, str>:
    if b == 0.0:
        return Err("division by zero")
    return Ok(a / b)
```

`Ok(value)` — the success case.
`Err(error)` — the failure case.

Consuming a `Result`:

```hmat
match divide(10.0, 2.0):
    Ok(result) -> print(f"Answer: {result}")
    Err(msg)   -> print(f"Error: {msg}")
```

### Helper Methods

```hmat
let result = divide(10.0, 2.0)

result.unwrap()              # returns value, panics if Err
result.unwrap_or(0.0)        # returns value, or default if Err
result.unwrap_or_else(|e| handle(e))    # returns value, or calls fn if Err
result.is_ok()               # bool
result.is_err()              # bool
result.map(|v| v * 2.0)      # transform the Ok value, pass Err through
result.map_err(|e| format(e)) # transform the Err value, pass Ok through
```

---

## The ? Operator

The `?` operator is syntactic sugar for "return early if this is an `Err`":

```hmat
fn process() -> Result<float, str>:
    let a = divide(10.0, 2.0)?    # returns Err if this is Err
    let b = divide(a, 3.0)?
    return Ok(b)
```

Without `?`, the same code would be:

```hmat
fn process() -> Result<float, str>:
    let a = match divide(10.0, 2.0):
        Ok(v) -> v
        Err(e) -> return Err(e)
    let b = match divide(a, 3.0):
        Ok(v) -> v
        Err(e) -> return Err(e)
    return Ok(b)
```

Use `?` everywhere. It's not just shorter — it makes the happy path obvious.

The enclosing function must return `Result` (or `Option`) for `?` to work.

---

## Option\<T\>

`Option<T>` is for values that may not exist — the safe replacement for `null`.

```hmat
fn first<T>(list: [T]) -> Option<T>:
    if list.is_empty():
        return None
    return Some(list[0])

match first([1, 2, 3]):
    Some(v) -> print(f"First: {v}")
    None    -> print("Empty list")
```

### Helper Methods

```hmat
let maybe = first([1, 2, 3])

maybe.unwrap()           # value or panic
maybe.unwrap_or(0)       # value or default
maybe.is_some()          # bool
maybe.is_none()          # bool
maybe.map(|v| v * 2)     # transform if Some, None stays None
maybe.and_then(|v| f(v)) # chain Option-returning functions
maybe.filter(|v| *v > 0) # None if predicate fails
```

`?` works with `Option` too — early-returns `None` if the value is `None`:

```hmat
fn first_doubled(list: [int]) -> Option<int>:
    let f = first(list)?    # returns None if list is empty
    return Some(f * 2)
```

---

## Custom Error Types

For real programs, define your own error types. Use an enum:

```hmat
enum AppError:
    NotFound(str)
    PermissionDenied
    IoError(str)

fn load_file(path: str) -> Result<str, AppError>:
    if not fs.exists(path):
        return Err(AppError::NotFound(path))
    if not fs.readable(path):
        return Err(AppError::PermissionDenied)
    return Ok(fs.read(path)?)
```

Pattern match on specific variants to handle them differently:

```hmat
match load_file("config.hm"):
    Ok(content)                        -> parse(content)
    Err(AppError::NotFound(p))         -> print(f"Missing: {p}")
    Err(AppError::PermissionDenied)    -> print("Access denied")
    Err(AppError::IoError(msg))        -> print(f"IO error: {msg}")
```

---

## Converting Between Error Types

When a function returns `Result<T, AppError>` but calls a function returning `Result<T, IoError>`,
use `.map_err()` to convert:

```hmat
fn load(path: str) -> Result<Config, AppError>:
    let content = fs.read(path)
        .map_err(|e| AppError::IoError(e.to_string()))?
    return Config.parse(content)
```

---

## Combining Multiple Results

When all operations must succeed:

```hmat
fn all_or_nothing() -> Result<(int, str), AppError>:
    let a = step_one()?
    let b = step_two()?
    return Ok((a, b))
```

When you want to collect all results (success or failure):

```hmat
let results: [Result<int, str>] = inputs.map(|x| process(x))
let (successes, failures) = results.partition_result()
```

---

## Panics

Panics are for **programming errors**, not expected failures. Use them only when the program
is in a state that should never happen:

```hmat
fn get_first(list: [int]) -> int:
    if list.is_empty():
        panic("get_first called with empty list — this is a bug")
    return list[0]
```

For all user-facing or recoverable errors, use `Result` or `Option` instead.

---

## When to Use What

| Situation                                    | Use                   |
|----------------------------------------------|-----------------------|
| Function that can fail with useful error info | `Result<T, E>`        |
| Value that might not exist                   | `Option<T>`           |
| Invariant that must hold — bug if broken     | `panic!`              |
| Need to propagate errors upward              | `?` operator          |
| Converting between error types               | `.map_err()`          |

---

## See Also

- [Syntax Guide](syntax-guide.md) — `match` patterns
- [Getting Started](getting-started.md) — introductory examples
- [AI Guide](ai-guide.md) — `AiError` and AI error handling
