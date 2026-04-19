# HMAT Language Specification v0.2
# Standard Library Overview
# Author: Hassan Zaib Hayat <hassanzaibhayatske@gmail.com>

---

## 1. Design Principles

The HMAT standard library is:
- **Curated, not comprehensive** — include what 90% of programs need, not everything
- **Consistent** — method names follow a single naming convention across all modules
- **Safe by default** — all public APIs return Result or Option where failure is possible
- **AI-aware** — stdlib modules are compatible with AI pipelines
- **Minimal core** — `hmat::core` is always available with zero imports

---

## 2. Module Hierarchy

```
hmat::
├── core/          # Always available — no import needed
│   ├── types      # Option, Result, primitives
│   ├── ops        # Operators, traits (Comparable, Clone, etc.)
│   ├── iter       # Iterators and iteration utilities
│   └── convert    # Type conversions (Into, From, as)
├── io/            # Input/output
│   ├── fs         # File system
│   ├── stdin      # Standard input
│   ├── stdout     # Standard output
│   └── path       # Path manipulation
├── collections/   # Data structures
│   ├── vec        # Dynamic arrays (alias: [T])
│   ├── map        # Hash maps (alias: {K:V})
│   ├── set        # Hash sets (alias: {T})
│   ├── deque      # Double-ended queue
│   ├── heap       # Priority queue
│   └── graph      # Graph structures
├── str/           # String utilities
│   ├── format     # Formatting
│   ├── parse      # Parsing
│   ├── regex      # Regular expressions
│   └── unicode    # Unicode utilities
├── math/          # Mathematics
│   ├── basic      # sqrt, pow, abs, floor, ceil, etc.
│   ├── stats      # mean, median, std, variance
│   ├── random     # Random number generation
│   └── matrix     # Matrix operations
├── net/           # Networking
│   ├── http       # HTTP client and server
│   ├── tcp        # TCP sockets
│   ├── udp        # UDP sockets
│   └── ws         # WebSockets
├── ai/            # AI-native constructs
│   ├── model      # Model loading and management
│   ├── chat       # ChatMessage, conversation management
│   ├── pipeline   # Pipeline composition utilities
│   └── embed      # Embedding utilities
├── async/         # Async primitives
│   ├── task       # Task spawning
│   ├── channel    # Channels (mpsc, broadcast)
│   ├── mutex      # Mutex, RwLock
│   └── timer      # Sleep, timeout, interval
├── time/          # Date and time
│   ├── instant    # High-resolution timer
│   ├── datetime   # Calendar types
│   └── duration   # Duration arithmetic
├── env/           # Environment
│   ├── args       # Command-line arguments
│   ├── vars       # Environment variables
│   └── process    # Process management
├── sys/           # System-level (requires @allow or unsafe)
│   ├── memory     # Manual memory management
│   ├── ffi        # Foreign function interface
│   └── signal     # OS signals
└── test/          # Testing utilities (test builds only)
    ├── assert     # Assertion macros
    ├── mock       # Mocking utilities
    └── bench      # Benchmarking
```

---

## 3. Core (Auto-Imported)

Everything in `hmat::core` is available without an import statement.

### 3.1 Built-in Functions

```hmat
print(value)                    # prints to stdout with newline
eprint(value)                   # prints to stderr with newline
input(prompt: str) -> str       # reads a line from stdin
panic(message: str)             # terminates program with error
assert(condition: bool)         # panics if condition is false
assert(condition, message: str) # panics with message if false
type_of(value) -> str           # returns type name as string
size_of<T>() -> int             # size of type in bytes
```

### 3.2 Core Traits (always in scope)

```hmat
trait Clone:    fn clone(self) -> Self
trait Copy:     # marker trait — bitwise copy allowed
trait Debug:    fn debug(self) -> str
trait Display:  fn display(self) -> str
trait Equals:   fn equals(self, other: Self) -> bool
trait Comparable: fn compare(self, other: Self) -> int
trait Hash:     fn hash(self) -> uint
trait Default:  fn default() -> Self
trait Into<T>:  fn into(self) -> T
trait From<T>:  fn from(value: T) -> Self
trait Serialize: fn to_json(self) -> str
trait Deserialize: fn from_json(data: str) -> Result<Self, ParseError>
```

### 3.3 Core Iterator Protocol

```hmat
trait Iterator<T>:
    fn next(mut self) -> Option<T>

    # Default implementations
    fn map<U>(self, f: fn(T) -> U) -> [U]
    fn filter(self, f: fn(T) -> bool) -> [T]
    fn reduce(self, f: fn(T, T) -> T) -> Option<T>
    fn fold<U>(self, init: U, f: fn(U, T) -> U) -> U
    fn collect() -> [T]
    fn count() -> int
    fn any(self, f: fn(T) -> bool) -> bool
    fn all(self, f: fn(T) -> bool) -> bool
    fn first(self) -> Option<T>
    fn last(self) -> Option<T>
    fn take(self, n: int) -> [T]
    fn skip(self, n: int) -> Iterator<T>
    fn zip<U>(self, other: Iterator<U>) -> Iterator<(T, U)>
    fn enumerate(self) -> Iterator<(int, T)>
    fn flat_map<U>(self, f: fn(T) -> [U]) -> [U]
    fn chain(self, other: Iterator<T>) -> Iterator<T>
    fn sort() -> [T] where T: Comparable
    fn sort_by(self, f: fn(T, T) -> int) -> [T]
    fn unique() -> [T] where T: Equals + Hash
    fn max() -> Option<T> where T: Comparable
    fn min() -> Option<T> where T: Comparable
    fn sum() -> T where T: Add + Default
```

---

## 4. hmat::io — File System

```hmat
import hmat::io::fs

# Reading
let content: str = fs.read("file.txt")?               # full file as string
let bytes: [byte] = fs.read_bytes("file.bin")?         # full file as bytes
let lines: [str] = fs.read_lines("file.txt")?          # line by line

# Writing
fs.write("file.txt", content)?                         # overwrite
fs.append("file.txt", content)?                        # append
fs.write_bytes("file.bin", bytes)?

# File operations
fs.exists("file.txt") -> bool
fs.delete("file.txt")?
fs.copy("src.txt", "dst.txt")?
fs.move("old.txt", "new.txt")?
fs.mkdir("dir/")?
fs.mkdir_all("a/b/c/")?
fs.list("dir/") -> Result<[str], IoError>              # list directory

# Path utilities
import hmat::io::path

let p = path.join("dir", "file.txt")     # "dir/file.txt"
let ext = path.extension("file.txt")     # "txt"
let stem = path.stem("file.txt")         # "file"
let dir = path.parent("dir/file.txt")    # "dir"
let abs = path.absolute("./file.txt")?
```

---

## 5. hmat::collections

### Dynamic Array ([T])

```hmat
let mut v: [int] = []
v.push(1)
v.push(2)
v.push(3)
v.pop()                  # Option<int>
v.insert(0, 99)          # insert at index
v.remove(0)              # remove at index
v.length()               # int
v.is_empty()             # bool
v.contains(2)            # bool
v.index_of(2)            # Option<int>
v.reverse()              # mutates in place
v.sort()                 # mutates, requires T: Comparable
v.sort_by(|a, b| a.compare(b))
v.slice(1, 3)            # [T] — elements 1..3
v.join(", ")             # str — requires T == str
v.clear()
v.extend(other_vec)
```

### Map ({K: V})

```hmat
let mut m: {str: int} = {}
m["key"] = 42
m.get("key")             # Option<int>
m.contains("key")        # bool
m.remove("key")          # Option<int>
m.length()               # int
m.keys()                 # [str]
m.values()               # [int]
m.entries()              # [(str, int)]
m.get_or_insert("key", 0)  # int — inserts default if missing
```

### Set ({T})

```hmat
let mut s: {str} = {}
s.add("a")
s.contains("a")          # bool
s.remove("a")
s.length()               # int
s.union(other)           # {T}
s.intersection(other)    # {T}
s.difference(other)      # {T}
```

---

## 6. hmat::str

```hmat
import hmat::str

# String methods (also available on str directly)
let s = "  Hello, World!  "

s.length()                    # 18
s.trim()                      # "Hello, World!"
s.trim_start()                # "Hello, World!  "
s.trim_end()                  # "  Hello, World!"
s.upper()                     # "  HELLO, WORLD!  "
s.lower()                     # "  hello, world!  "
s.contains("World")           # bool
s.starts_with("  H")         # bool
s.ends_with("!  ")           # bool
s.replace("World", "HMAT")    # str
s.split(", ")                 # [str]
s.split_once(", ")            # Option<(str, str)>
s.lines()                     # [str]
s.chars()                     # [char]
s.bytes()                     # [byte]
s.parse::<int>()              # Result<int, ParseError>
s.parse::<float>()            # Result<float, ParseError>
s.index_of("World")           # Option<int>
s.slice(2, 7)                 # str
s.repeat(3)                   # str
s.is_empty()                  # bool
s.pad_left(20, ' ')           # str
s.pad_right(20, '-')          # str
```

---

## 7. hmat::net::http

```hmat
import hmat::net::http

# GET
let response = await http.get("https://api.example.com/data")?
let body: str = response.text()?
let json: MyType = response.json::<MyType>()?
let status = response.status    # int

# POST
let response = await http.post(
    "https://api.example.com/submit",
    body: "{ \"key\": \"value\" }",
    headers: { "Content-Type": "application/json" }
)?

# HTTP client with config
let client = http.Client:
    timeout: 30
    base_url: "https://api.example.com"
    headers: { "Authorization": f"Bearer {env("API_TOKEN")}" }

let response = await client.get("/endpoint")?
```

---

## 8. hmat::async

```hmat
import hmat::async

# Spawn a task (runs concurrently)
let task = async::spawn(async fn():
    await some_work()
)
let result = await task?

# Join multiple tasks (wait for all)
let (a, b, c) = await async::join(task1, task2, task3)?

# Race (first to complete wins)
let result = await async::race(fast_task, slow_task)?

# Channel (message passing between tasks)
let (sender, receiver) = async::channel::<str>()

async::spawn(async fn():
    sender.send("hello")?
    sender.send("world")?
    sender.close()
)

for msg in receiver:
    print(msg)

# Sleep
await async::sleep(1.5)    # seconds

# Timeout
let result = await async::timeout(5.0, slow_operation())?
# returns Err(AiError.Timeout) if slow_operation takes > 5 seconds

# Mutex
let data = async::Mutex::new([1, 2, 3])
let guard = await data.lock()
guard.push(4)
# guard released when it goes out of scope
```

---

## 9. hmat::math

```hmat
import hmat::math

math.sqrt(16.0)         # 4.0
math.pow(2.0, 10.0)     # 1024.0
math.abs(-5)            # 5
math.floor(3.7)         # 3.0
math.ceil(3.2)          # 4.0
math.round(3.5)         # 4.0
math.min(3, 5)          # 3
math.max(3, 5)          # 5
math.clamp(15, 0, 10)   # 10
math.log(100.0, 10.0)   # 2.0
math.log2(8.0)          # 3.0
math.ln(math.E)         # 1.0
math.sin(math.PI / 2)   # 1.0
math.cos(0.0)           # 1.0

math.PI     # 3.14159...
math.E      # 2.71828...
math.INF    # infinity
math.NAN    # not a number

# Statistics (from [T] where T is numeric)
let nums = [1, 2, 3, 4, 5]
nums.sum()       # 15
nums.mean()      # 3.0
nums.median()    # 3.0
nums.std()       # standard deviation
nums.variance()

# Random
import hmat::math::random

random.int(1, 10)           # random int in [1, 10]
random.float(0.0, 1.0)     # random float in [0.0, 1.0]
random.choice(list)         # random element
random.shuffle(list)        # shuffle in place
random.seed(42)             # deterministic seed
```

---

## 10. hmat::test

```hmat
import hmat::test

@test
fn test_addition():
    assert(1 + 1 == 2)
    assert_eq(1 + 1, 2)
    assert_ne(1 + 1, 3)

@test
fn test_string():
    let s = "hello"
    assert_eq(s.upper(), "HELLO")
    assert(s.contains("ell"))

@test
fn test_error():
    let result = divide(10.0, 0.0)
    assert_err(result)
    assert_err_kind(result, DivisionError.ZeroDivision)

@test
fn test_approx():
    assert_approx(math.sqrt(2.0), 1.414, tolerance: 0.001)

@bench
fn bench_sort():
    let nums = random.ints(10_000)
    nums.sort()

# Run tests
# hmatc test          — all tests
# hmatc test lexer    — tests matching "lexer"
# hmatc bench         — benchmarks
```

---

## 11. Naming Conventions (stdlib enforced by hfmt)

| Element         | Convention          | Example                  |
|-----------------|---------------------|--------------------------|
| Functions       | `snake_case`        | `read_file`, `parse_int` |
| Methods         | `snake_case`        | `to_upper`, `is_empty`   |
| Types           | `PascalCase`        | `HttpClient`, `ChatMessage` |
| Constants       | `SCREAMING_SNAKE`   | `MAX_CONNECTIONS`, `PI`  |
| Modules         | `snake_case`        | `hmat::net::http`        |
| Errors          | `PascalCase`        | `IoError`, `ParseError`  |
| Error variants  | `PascalCase`        | `IoError.NotFound`       |
| Traits          | `PascalCase`        | `Comparable`, `Serialize`|
| Generic params  | Single uppercase    | `T`, `K`, `V`, `E`       |

---

## 12. Error Types (stdlib defined)

```hmat
enum IoError:
    NotFound(str)
    PermissionDenied(str)
    AlreadyExists(str)
    IsDirectory(str)
    NotDirectory(str)
    UnexpectedEof
    Other(str)

enum ParseError:
    InvalidFormat(str)
    Overflow
    Underflow
    UnexpectedEnd

enum HttpError:
    NetworkError(str)
    Timeout
    InvalidUrl(str)
    StatusError { code: int, body: str }
    InvalidResponse(str)

enum AiError:
    NetworkError(str)
    Timeout
    RateLimited { retry_after: int }
    InvalidResponse(str)
    ContextLengthExceeded
    Unauthorized
    ModelNotFound(str)
```
