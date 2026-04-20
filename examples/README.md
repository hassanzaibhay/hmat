# HMAT Examples

Working programs that demonstrate the language. Read them in order if you're new to HMAT.

---

## Examples

### [hello_world.hm](hello_world.hm)

The milestone example. When this compiles and runs, Phase 1 is done.

```
Demonstrates: fn, print
Phase: 1 (Hello World)
Status: pending compiler
```

### [core_language.hm](core_language.hm)

A tour of the core language: variables, functions, structs, enums, pattern matching,
generics, closures, iterators, and error handling.

```
Demonstrates: let, mut, fn, struct, enum, match, for, closures, Result, Option, ?
Phase: 2 (Core Language)
Status: pending compiler
```

### [ai_chat.hm](ai_chat.hm)

The AI-native showcase. Demonstrates everything that makes HMAT unique:
AI model declarations, multi-turn chat, structured extraction, pipelines, and parallel calls.

```
Demonstrates: ai model, await, ChatMessage, extract::<T>, pipeline, join
Phase: 3 (AI Native)
Status: pending compiler + AI runtime
```

---

## Running an Example

Once `hmatc` is available:

```bash
# Compile
hmatc examples/hello_world.hm -o hello

# Run
./hello
```

For AI examples, set your API key first:

```bash
export ANTHROPIC_API_KEY=your_key_here
hmatc examples/ai_chat.hm -o ai_chat
./ai_chat
```

---

## Reading the Examples

The examples are written to be read, not just run. Each one:

- Has a header comment explaining what it demonstrates and its phase
- Uses comments sparingly — only where the *why* is non-obvious
- Follows realistic patterns, not toy code

If you find a pattern that's unclear, that's a documentation bug. File an issue.

---

## See Also

- [Getting Started](../docs/getting-started.md) — first steps
- [Syntax Guide](../docs/syntax-guide.md) — full language reference
- [AI Guide](../docs/ai-guide.md) — AI-native features
