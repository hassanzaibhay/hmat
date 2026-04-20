# Async Guide

HMAT has first-class async/await support. Async is not an afterthought — it's how HMAT
handles all IO, including AI calls, network requests, and file operations.

---

## The Basics

Mark a function as async with the `async` keyword:

```hmat
async fn fetch(url: str) -> Result<str, HttpError>:
    let response = await http.get(url)
    return response.text()
```

Call it with `await`:

```hmat
async fn main():
    let html = await fetch("https://example.com")?
    print(html.len())
```

`await` suspends the current function until the async operation completes. While waiting,
other async tasks can run. No threads required.

---

## Why Async?

- AI calls are network operations — they take hundreds of milliseconds.
- Without async, your program blocks while waiting, wasting CPU.
- With async, many operations can be in-flight simultaneously.

This is especially important for AI programs that make multiple model calls per request.

---

## Async Functions

Any function that uses `await` must itself be declared `async`:

```hmat
# Fine — no await inside
fn compute(x: int) -> int:
    x * 2

# Must be async — uses await
async fn get_answer() -> str:
    await assistant.chat("What is 2 + 2?")
```

`async` propagates: if `get_answer` is async, every function that calls it with `await`
must also be async.

---

## The `main` Entry Point

If your program does async work, declare `main` as async:

```hmat
async fn main():
    let reply = await assistant.chat("Hello!")
    print(reply)
```

The runtime sets up the event loop before calling `main`. You don't need to configure it.

---

## Running Tasks in Parallel

Use `join` to run multiple async operations at the same time:

```hmat
from hmat::async import join

async fn main():
    match await join(
        fetch("https://api.example.com/users"),
        fetch("https://api.example.com/products"),
        fetch("https://api.example.com/orders")
    ):
        Ok((users, products, orders)):
            process(users, products, orders)
        Err(e):
            print(f"A request failed: {e}")
```

`join` waits for all tasks. If any fails, the first error is returned.

### `race` — First to Succeed

```hmat
from hmat::async import race

# Use the fastest of three mirror servers
match await race(
    fetch("https://mirror1.example.com/data"),
    fetch("https://mirror2.example.com/data"),
    fetch("https://mirror3.example.com/data")
):
    Ok(data) -> process(data)
    Err(e)   -> print(f"All mirrors failed: {e}")
```

`race` returns the first successful result and cancels the rest.

---

## Spawning Background Tasks

```hmat
from hmat::async import spawn

async fn main():
    let handle = spawn(background_work())    # starts immediately, doesn't block
    
    # do other things...
    do_something_else()

    let result = await handle    # wait for background task to finish
```

Use `spawn` when a task can run completely independently and you don't need the result
immediately.

---

## Async in Loops

```hmat
async fn process_all(items: [str]):
    # Sequential — each waits for the previous
    for item in items:
        let result = await process_one(item)?
        print(result)

    # Parallel — all at once
    let tasks = items.map(|item| process_one(item))
    let results = await join_all(tasks)?
```

For large lists where you want parallelism but not an unlimited number of concurrent tasks:

```hmat
from hmat::async import Pool

async fn batch_process(items: [str]) -> Result<[str], AppError>:
    let pool = Pool::new(max_concurrent: 10)
    return await pool.map(items, |item| process_one(item))
```

---

## Timeouts

```hmat
from hmat::async import timeout

async fn safe_fetch(url: str) -> Result<str, AppError>:
    match await timeout(5000, fetch(url)):    # 5 second timeout
        Ok(data)           -> Ok(data)
        Err(TimeoutError)  -> Err(AppError::Timeout)
        Err(e)             -> Err(e)
```

---

## Async Traits

Traits can have async methods:

```hmat
trait DataSource:
    async fn fetch(self, query: str) -> Result<[Row], DbError>

impl DataSource for PostgresDb:
    async fn fetch(self, query: str) -> Result<[Row], DbError>:
        # ...
```

---

## Common Patterns

### Retry with Backoff

```hmat
from hmat::async import sleep

async fn with_retry<T>(
    operation: async fn() -> Result<T, AppError>,
    max_attempts: int
) -> Result<T, AppError>:
    let mut attempt = 0
    while attempt < max_attempts:
        match await operation():
            Ok(result) -> return Ok(result)
            Err(AppError::RateLimited { retry_after }):
                await sleep(retry_after * 1000)    # ms
                attempt += 1
            Err(e) -> return Err(e)
    return Err(AppError::MaxRetriesExceeded)
```

### Fan-out / Fan-in

```hmat
async fn summarize_all(documents: [str]) -> Result<[str], AiError>:
    # Fan out: all summarizations in parallel
    let summaries = await join_all(
        documents.map(|doc| assistant.summarize(doc))
    )?
    return Ok(summaries)
```

---

## See Also

- [AI Guide](ai-guide.md) — async AI calls
- [Error Handling Guide](error-handling.md) — `?` with async functions
- [Syntax Guide](syntax-guide.md) — `async`, `await` keywords
