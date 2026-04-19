# HMAT Language Specification v0.2
# AI-Native Constructs
# Author: Hassan Zaib Hayat <hassanzaibhayatske@gmail.com>

---

## 1. Philosophy

HMAT is the first general-purpose language where AI is not a library — it is a **language construct.**

In most languages, calling an AI model looks like calling an HTTP endpoint.
In HMAT, the compiler understands AI models: their inputs are typed, their outputs are typed,
their errors are handled like any other Result, and the toolchain can reason about AI calls
at compile time.

This enables:
- **Type-safe AI calls** — wrong output type is a compile error, not a runtime surprise
- **Auditable AI usage** — the compiler can list every AI call in your program
- **Safe eval** — AI-generated code is a typed value (`HmatCode`), not an `eval(string)`
- **Sandboxed models** — models cannot access filesystem/network unless explicitly granted
- **Pipeline composition** — AI stages compose with regular functions cleanly

---

## 2. Model Declaration

### 2.1 Basic Declaration

```hmat
ai model NAME = load(MODEL_STRING)
```

`MODEL_STRING` is a string literal identifying the model provider and name.
Format: `"provider/model-name"` or `"provider/model-name@version"`

```hmat
ai model assistant = load("anthropic/claude-3-5-sonnet")
ai model fast = load("anthropic/claude-haiku-4-5")
ai model gpt = load("openai/gpt-4o")
ai model local = load("ollama/llama3.2")
ai model custom = load("http://localhost:8080/v1")
```

### 2.2 Model Configuration

```hmat
ai model assistant = load("anthropic/claude-3-5-sonnet"):
    temperature: 0.7        # float, 0.0–2.0
    max_tokens: 4096        # int
    timeout: 30             # int, seconds
    system: "You are a helpful HMAT assistant."  # str
```

### 2.3 Scoped Models

Models declared at module level are module-level. Models declared inside a function are local.

```hmat
# Module level — available everywhere in this module
ai model shared = load("anthropic/claude-3-5-sonnet")

fn process(text: str) -> str:
    # Function level — created fresh each call
    ai model local = load("openai/gpt-4o-mini")
    return await local.summarize(text)
```

### 2.4 Model Visibility

```hmat
pub ai model assistant = load("anthropic/claude-3-5-sonnet")  # exported
ai model internal = load("anthropic/claude-haiku-4-5")        # module-private
```

---

## 3. AI Method Calls

AI model calls are always `async` and return `Result<T, AiError>`.

### 3.1 Chat / Completion

```hmat
# Basic chat — returns str
let reply: str = await assistant.chat("What is the capital of France?")?

# With system context
let reply = await assistant.chat("Summarize this", system: "Be brief.")?

# Multi-turn
let messages: [ChatMessage] = [
    ChatMessage { role: "user", content: "Hello" },
    ChatMessage { role: "assistant", content: "Hi! How can I help?" },
    ChatMessage { role: "user", content: "What is HMAT?" }
]
let reply = await assistant.chat_history(messages)?
```

### 3.2 Typed Output Methods

These methods parse the model's output into typed HMAT values:

```hmat
# Summarize — returns str
let summary: str = await assistant.summarize(long_document)?

# Generate code — returns HmatCode (typed, safe)
let code: HmatCode = await assistant.generate_code("sort a list by frequency")?

# Extract structured data — returns T where T: Deserialize
struct PersonInfo:
    name: str
    age: int
    occupation: str

let info: PersonInfo = await assistant.extract::<PersonInfo>(bio_text)?

# Classify — returns one of a set of labels
let sentiment: str = await assistant.classify(
    text,
    labels: ["positive", "negative", "neutral"]
)?

# Embed — returns a vector
let embedding: [float] = await assistant.embed("hello world")?
```

### 3.3 Error Handling

All AI calls return `Result<T, AiError>`:

```hmat
enum AiError:
    NetworkError(str)
    Timeout
    RateLimited { retry_after: int }
    InvalidResponse(str)
    ContextLengthExceeded
    Unauthorized
    ModelNotFound(str)

# Always handle AI errors
match await assistant.chat("Hello"):
    Ok(reply) -> print(reply)
    Err(AiError.Timeout) -> print("Model timed out, try again")
    Err(AiError.RateLimited { retry_after }) -> sleep(retry_after)
    Err(e) -> print(f"AI error: {e}")

# Or propagate with ?
fn ask(question: str) -> Result<str, AiError>:
    let reply = await assistant.chat(question)?
    return Ok(reply)
```

---

## 4. The `await` Keyword

HMAT supports both prefix and postfix `await`. They are equivalent.

```hmat
# Prefix (recommended for clarity)
let result = await some_async_fn()

# Postfix (for chaining)
let result = some_async_fn().await

# Chaining postfix
let processed = fetch_data(url)
    .await
    .map(|data| process(data))
    .await?
```

`await` can only be used inside `async` functions.
Using `await` in a non-async function is a compile error (E301).

```
error[E301]: `await` used in non-async function
  --> main.hm:5:15
   |
3  |     fn fetch(url: str) -> str:
   |     -- this function is not async
5  |         let r = await http.get(url)
   |                 ^^^^^ `await` not allowed here
   |
   = help: add `async` to the function signature: `async fn fetch(url: str) -> str:`
```

---

## 5. HmatCode — Safe AI-Generated Code

AI models can generate HMAT code. The result is typed as `HmatCode`, not a raw string.

```hmat
let code: HmatCode = await assistant.generate_code("
    Write a function that takes a list of strings
    and returns them sorted by length, then alphabetically
")?

# Inspect the generated code
print(code.source)           # the HMAT source as a string
print(code.functions)        # list of function names generated

# Execute it (ONLY in unsafe context)
# SAFETY: code was generated from a controlled prompt with no user input
unsafe ai:
    let sorted = eval(code, input: ["banana", "apple", "cherry", "fig"])
```

`eval()` inside `unsafe ai` compiles and runs `HmatCode` in a sandboxed environment.
The sandbox has no filesystem access, no network access, no AI calls.
The sandbox is enforced at the compiler/runtime level, not by trust.

---

## 6. Pipelines

Pipelines compose AI and regular functions into readable data transformation chains.

### 6.1 Pipeline Declaration

```hmat
pipeline NAME:
    STAGE -> STAGE -> STAGE -> ...
```

Each stage is either:
- A function name
- A method call `object.method`
- An AI method call (automatically async)

```hmat
ai model nlp = load("anthropic/claude-3-5-sonnet")

pipeline analyze_feedback:
    input -> clean_text -> nlp.classify(labels: ["bug", "feature", "praise"]) -> output

pipeline summarize_documents:
    load_documents(path) -> chunk(size: 2000) -> nlp.summarize -> combine -> output
```

### 6.2 Running a Pipeline

```hmat
# Single input
let result = await analyze_feedback.run(user_feedback)?

# Batch input
let results: [str] = await summarize_documents.run_batch(doc_paths)?

# Streaming
await analyze_feedback.run_stream(feedback_stream, |result| print(result))?
```

### 6.3 Pipeline Composition

```hmat
pipeline preprocess:
    input -> normalize -> tokenize

pipeline classify:
    preprocess -> embed -> model.classify

# Pipelines compose
pipeline full_pipeline:
    classify -> output
```

---

## 7. AI Security Rules (Enforced by Compiler)

These rules are enforced at compile time and cannot be bypassed:

### Rule 1: No Hardcoded API Keys

```hmat
# ERROR E302: hardcoded API key detected
ai model m = load("openai/gpt-4o"):
    api_key: "sk-abc123..."

# CORRECT: use environment or config
ai model m = load("openai/gpt-4o"):
    api_key: env("OPENAI_API_KEY")
```

### Rule 2: Model Identifiers Must Be String Literals

Dynamic model selection requires an explicit allow:

```hmat
# ERROR E303: model identifier must be a string literal
let model_name = get_user_input()
ai model m = load(model_name)   # could load anything — not allowed

# Allowed: static selection
fn get_model(fast: bool) -> Model:
    if fast:
        return load("anthropic/claude-haiku-4-5")
    else:
        return load("anthropic/claude-3-5-sonnet")
```

### Rule 3: `eval()` Requires `unsafe ai`

```hmat
let code: HmatCode = await assistant.generate_code("...")?

# ERROR E304: eval() requires unsafe ai block
eval(code)

# CORRECT
unsafe ai:
    eval(code)
```

### Rule 4: File System Access Requires Explicit Grant

```hmat
# By default, AI models cannot read files
ai model m = load("anthropic/claude-3-5-sonnet")
let content = fs.read("data.txt")
let summary = await m.summarize(content)?  # OK — we read the file, not the model

# If a pipeline stage reads files, it must be annotated
@allow(fs.read)
pipeline document_analyzer:
    load_file(path) -> m.analyze -> output
```

---

## 8. Async and AI

All AI calls are async. Functions that call AI models must be async.

```hmat
# This propagates: if you call an async fn, you must be async
async fn analyze(text: str) -> Result<str, AiError>:
    let result = await assistant.chat(f"Analyze: {text}")?
    return Ok(result)

# Main can be async
async fn main():
    match await analyze("HMAT is great"):
        Ok(result) -> print(result)
        Err(e) -> print(f"Error: {e}")
```

### Parallel AI Calls

```hmat
# Sequential (slower)
let a = await model.chat("Question 1")?
let b = await model.chat("Question 2")?

# Parallel (faster) — both calls happen concurrently
let (a, b) = await join(
    model.chat("Question 1"),
    model.chat("Question 2")
)?

# Fan-out to multiple models
let (result1, result2) = await join(
    model_a.chat("Compare A"),
    model_b.chat("Compare B")
)?
```

---

## 9. AI Standard Library

These are in the `hmat::ai` standard library module:

```hmat
from hmat::ai import ChatMessage, HmatCode, AiError, join, race

# join — await multiple futures, return tuple of results
# race — await first to complete, cancel others
# ChatMessage — structured message for multi-turn conversations
# HmatCode — typed representation of AI-generated HMAT code
# AiError — all AI error variants
```

---

## 10. AI Error Reference

| Error                     | Meaning                                 | Common Fix                          |
|---------------------------|-----------------------------------------|-------------------------------------|
| `AiError.NetworkError`    | Cannot reach the model provider         | Check connectivity, retry            |
| `AiError.Timeout`         | Model didn't respond in time             | Increase timeout or use faster model |
| `AiError.RateLimited`     | Too many requests                        | Implement backoff, check retry_after |
| `AiError.InvalidResponse` | Model returned unparseable output        | Check your prompt, add constraints   |
| `AiError.ContextLengthExceeded` | Input too long for model           | Chunk input, use a larger context model |
| `AiError.Unauthorized`    | Invalid or missing API key               | Check `env("API_KEY")` configuration |
| `AiError.ModelNotFound`   | Model identifier doesn't exist           | Check the model string format        |

---

## 11. Compile-Time AI Analysis

The compiler can, when `--analyze-ai` flag is passed, emit:

```bash
hmatc --analyze-ai main.hm

AI Usage Report for main.hm:
  Models loaded: 2
    - anthropic/claude-3-5-sonnet (module level)
    - openai/gpt-4o-mini (fn: process_feedback, line 42)
  AI calls: 5
    - line 15: assistant.chat(str) -> Result<str, AiError>
    - line 28: assistant.extract::<UserProfile>(str) -> Result<UserProfile, AiError>
    - line 42: local.summarize(str) -> Result<str, AiError>
    - line 67: nlp.classify(str, labels: [str]) -> Result<str, AiError>
    - line 89: gpt.generate_code(str) -> Result<HmatCode, AiError>
  Unsafe AI blocks: 1
    - line 91: eval(HmatCode) — review required
  Hardcoded keys: 0 ✅
  All errors handled: yes ✅
```
