# HMAT Language Specification v0.3
# AI-Native Constructs
# Author: Hassan Zaib Hayat <hassanzaibhayatske@gmail.com>

---

## 1. Philosophy

HMAT is the first language where AI is a language primitive — not a library.

In other languages, calling an AI model is indistinguishable from an HTTP call.
In HMAT, the compiler understands AI: validates model identifiers at compile time,
types AI outputs, enforces error handling, and audits AI usage.

---

## 2. Model Declaration

```hmat
# Declare an AI model — 'ai' keyword, unique to HMAT
ai assistant = model("anthropic/claude-3-5-sonnet")

# With configuration
ai gpt = model("openai/gpt-4o"):
    temperature: 0.7
    max_tokens: 2048
    system: "You are an HMAT assistant."

# Supported providers: anthropic, openai, ollama, http (custom endpoint)
ai local = model("ollama/llama3.2")
ai custom = model("http://localhost:8080/v1")
```

`model(...)` takes a string literal — not a variable. Dynamic model selection
requires an explicit `unsafe ai` block (security rule).

---

## 3. AI Method Calls

All AI calls are async. The compiler wraps them automatically in async context.
All return `T or Fail` — errors must be handled.

### Core Methods

```hmat
# ask — conversational response
reply: str = assistant.ask("What is HMAT?")
reply = assistant.ask("Summarize this", system: "Be brief.")

# think — longer reasoning / analysis
analysis: str = assistant.think(long_document)

# gen — generate typed HMAT code
code: GenCode = assistant.gen("sort a list by frequency")

# extract — parse structured data from text
shape PersonInfo:
    name: str
    age: int
    occupation: str

info: PersonInfo = assistant.extract[PersonInfo](bio_text)

# classify — label text
sentiment: str = assistant.classify(text, labels: ["positive", "negative", "neutral"])

# embed — vector embedding
vec: [float] = assistant.embed("hello world")
```

### Multi-Turn Conversation

```hmat
from hmat.ai import Msg

history: [Msg] = []
history.push(Msg { role: "user", content: "Hello" })
history.push(Msg { role: "assistant", content: "Hi! How can I help?" })

reply = assistant.chat(history, "What is HMAT?")
```

---

## 4. Error Handling for AI Calls

AI calls return `T or Fail`. Handle with `or` or `on`:

```hmat
# Default on failure
reply = assistant.ask("Hello") or "AI unavailable"

# Full handling
on assistant.ask("Hello"):
    str as reply => print(reply)
    Fail as e    => on e:
        AiError.Timeout       => print("timed out")
        AiError.RateLimited   => sleep(e.retry_after)
        _                     => print("error: {e}")
```

### AI Error Types

```hmat
type AiError:
    NetworkError(msg: str)
    Timeout
    RateLimited(retry_after: int)
    InvalidResponse(msg: str)
    ContextLengthExceeded
    Unauthorized
    ModelNotFound(name: str)
```

---

## 5. GenCode — Safe AI-Generated Code

AI can generate HMAT code. The result is `GenCode` — a typed value, not a raw string.

```hmat
code: GenCode = assistant.gen("
    Write a function that counts word frequency in a string
")

# Inspect without running
print(code.source)        # HMAT source as str
print(code.functions)     # list of generated function names

# Execute — only in unsafe ai block
# SAFETY: prompt is hardcoded, no user input
unsafe ai:
    result = eval(code, input: "hello world hello")
```

`eval()` runs in a sandbox with no filesystem/network/AI access.

---

## 6. Flow (Pipelines)

Flows compose AI and regular functions into readable data pipelines.

```hmat
ai nlp = model("anthropic/claude-haiku-4-5")

# Declare a flow
flow analyze_feedback:
    input => clean_text => nlp.classify(labels: ["bug", "feature", "praise"]) => output

flow summarize:
    load_docs(path) => chunk(size: 2000) => nlp.think => combine => output

# Run a flow
result = analyze_feedback.run(user_input) or "analysis failed"

# Batch
results: [str] = summarize.run_batch(doc_paths) or []

# Flows compose
flow full_pipeline:
    analyze_feedback => summarize => output
```

---

## 7. Parallel AI Calls

```hmat
from hmat.async import join

# Sequential (slower)
a = assistant.ask("Q1")
b = assistant.ask("Q2")

# Parallel (faster) — both happen concurrently
(a, b) = join(assistant.ask("Q1"), assistant.ask("Q2")) or ("", "")
```

---

## 8. Security Rules (Compiler-Enforced)

### Rule 1: No Hardcoded API Keys

```hmat
# ERROR E302: hardcoded API key
ai m = model("openai/gpt-4o"):
    api_key: "sk-abc123"   # compiler rejects this

# Correct
ai m = model("openai/gpt-4o"):
    api_key: env("OPENAI_API_KEY")
```

### Rule 2: Model Identifiers Must Be Literals

```hmat
# ERROR E303: dynamic model identifier
name = get_user_input()
ai m = model(name)   # rejected — could load anything

# OK: static selection
ai m = model("anthropic/claude-3-5-sonnet")
```

### Rule 3: eval() Requires unsafe ai

```hmat
code: GenCode = assistant.gen("...")

eval(code)           # ERROR E304 — requires unsafe ai

unsafe ai:           # OK
    eval(code)
```

### Rule 4: Filesystem Access Requires @allow

```hmat
# By default, AI models cannot read/write files
@allow(fs.read)
flow document_analyzer:
    load_file(path) => assistant.think => output
```

---

## 9. Compile-Time AI Analysis

```bash
hmatc --analyze-ai main.hm

AI Usage Report:
  Models: 2
    - anthropic/claude-3-5-sonnet (module level)
    - openai/gpt-4o-mini (fn: process, line 42)
  Calls: 4 (ask: 2, gen: 1, classify: 1)
  Unsafe AI blocks: 1 — review required (line 91)
  Hardcoded keys: 0 ✅
  Unhandled errors: 0 ✅
```
