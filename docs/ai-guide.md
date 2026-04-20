# AI Guide

HMAT is the first general-purpose language where AI is a first-class citizen.
Not a library. Not a framework. A language construct.

This guide covers everything the language provides: model declarations, AI calls,
structured extraction, pipelines, and error handling.

---

## The Idea

Most languages treat AI as an afterthought — you import an SDK, manage HTTP, parse JSON,
handle rate limits manually. HMAT makes AI as natural as calling a function.

```hmat
ai model assistant = load("anthropic/claude-3-5-sonnet")

async fn main():
    let reply = await assistant.chat("Explain HMAT in one sentence.")
    print(reply)
```

The language knows what an AI model is. The compiler validates model declarations.
The runtime handles retries, rate limits, and serialization.

---

## Model Declaration

```hmat
ai model name = load("provider/model-id")
```

With configuration:

```hmat
ai model assistant = load("anthropic/claude-3-5-sonnet"):
    temperature: 0.7
    max_tokens: 1024
    system: "You are a helpful assistant. Be concise."
```

Supported providers (Phase 3):

| Provider    | Example model ID                     |
|-------------|--------------------------------------|
| Anthropic   | `anthropic/claude-3-5-sonnet`        |
| OpenAI      | `openai/gpt-4o`                      |
| Ollama      | `ollama/llama3`                      |

Model declarations are top-level or local to a function. They are not expressions.

---

## Basic AI Calls

All AI calls are async. Call them with `await`:

```hmat
# Chat — free-form response
let reply: str = await assistant.chat("What is 2 + 2?")

# Summarize — shortens a document
let summary: str = await assistant.summarize(long_document)

# Classify — categorizes input
let label: str = await analyzer.classify(
    text,
    labels: ["positive", "negative", "neutral"]
)

# Embed — semantic vector
let vector: [float] = await embedder.embed("HMAT programming language")
```

---

## Structured Output

Use `extract::<T>()` when you want a typed struct instead of raw text:

```hmat
struct CodeReview:
    has_bugs: bool
    suggestions: [str]
    quality_score: int

async fn review(code: str) -> Result<CodeReview, AiError>:
    let review: CodeReview = await assistant.extract::<CodeReview>(
        f"Review this code and return structured feedback:\n\n{code}"
    )?
    return Ok(review)
```

The compiler generates a JSON schema from the struct. The model is prompted to return
conforming JSON. The runtime validates and deserializes it.

---

## Error Handling

AI calls return `Result<T, AiError>`. Always handle errors:

```hmat
match await assistant.chat("Hello"):
    Ok(reply)                      -> print(reply)
    Err(AiError::Timeout)          -> print("Model timed out")
    Err(AiError::RateLimited { retry_after }) ->
        print(f"Rate limited — retry in {retry_after}s")
    Err(AiError::InvalidResponse(msg)) ->
        print(f"Bad response: {msg}")
    Err(e) -> print(f"Unknown error: {e}")
```

The `?` operator works — propagate errors to the caller when you don't need to handle them locally:

```hmat
async fn pipeline(text: str) -> Result<str, AiError>:
    let summary = await assistant.summarize(text)?
    let sentiment = await analyzer.classify(summary, labels: ["good", "bad"])?
    return Ok(f"{sentiment}: {summary}")
```

---

## Multi-Turn Conversations

Use `ChatMessage` for conversation history:

```hmat
from hmat::ai import ChatMessage

let history: [ChatMessage] = []

history.push(ChatMessage { role: "user", content: "What is HMAT?" })

match await assistant.chat_history(history):
    Ok(reply):
        history.push(ChatMessage { role: "assistant", content: reply })
        print(reply)
    Err(e): print(f"Error: {e}")
```

---

## Pipelines

Pipelines compose AI operations into reusable data flows:

```hmat
ai model analyzer = load("anthropic/claude-haiku-4-5"):
    temperature: 0.0    # deterministic classification

pipeline analyze_sentiment:
    input -> analyzer.classify(labels: ["positive", "negative", "neutral"]) -> output
```

Run a pipeline:

```hmat
async fn process(text: str):
    match await analyze_sentiment.run(text):
        Ok(label) -> print(f"{text}: {label}")
        Err(e)    -> print(f"Failed: {e}")
```

Multi-stage pipelines:

```hmat
pipeline full_analysis:
    input
    -> summarizer.summarize()
    -> analyzer.classify(labels: ["positive", "negative", "neutral"])
    -> output
```

Each stage receives the output of the previous. Stages can be AI models or regular functions.

---

## Parallel AI Calls

Sequential calls waste time when the results are independent. Use `join` for parallel execution:

```hmat
from hmat::async import join

match await join(
    assistant.chat("In one word, describe Python."),
    assistant.chat("In one word, describe Rust."),
    assistant.chat("In one word, describe HMAT.")
):
    Ok((python, rust, hmat)):
        print(f"Python: {python}")
        print(f"Rust:   {rust}")
        print(f"HMAT:   {hmat}")
    Err(e):
        print(f"One call failed: {e}")
```

`join` waits for all calls. If any fails, the error propagates. Use `race` if you want
the first to succeed.

---

## Security Rules

The compiler enforces safety rules for AI code:

- **No hardcoded API keys.** Keys must come from environment variables or config files.
  The compiler rejects string literals that look like API keys.
- **Model IDs must be literals** or named constants — not dynamic strings.
  This prevents prompt injection attacks via model ID manipulation.
- **AI-generated code** requires explicit `eval()` inside an `unsafe` block.

```hmat
# This is rejected — potential key leak
ai model bad = load("anthropic/claude-3-5-sonnet"):
    api_key: "sk-ant-abc123..."    # compiler error: hardcoded key

# This is correct
ai model good = load("anthropic/claude-3-5-sonnet"):
    api_key: env("ANTHROPIC_API_KEY")
```

---

## The `HmatCode` Type

When an AI generates HMAT code, the result has the type `HmatCode` — not plain `str`:

```hmat
let generated: HmatCode = await assistant.generate_code(
    "Write a function that sorts a list by frequency"
)

# To execute it, you must explicitly opt in
unsafe:
    eval(generated)
```

This makes AI-generated code execution visible and auditable.

---

## See Also

- [AI Constructs Spec](../spec/0.2/ai-constructs.md) — formal specification
- [Error Handling Guide](error-handling.md) — handling `AiError`
- [Async Guide](async-guide.md) — `await`, `join`, async functions
- [Example: AI Chat](../examples/ai_chat.hm) — full working example
