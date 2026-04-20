//! Lexer throughput benchmarks.
//!
//! Baseline target: ≥ 50 MB/s on a modern laptop.
//! Run with: `cargo bench --bench lexer_bench`
//! HTML report written to: `target/criterion/`

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use hmatc::lexer::tokenize;

/// Build a synthetic HMAT source of approximately `target_bytes` bytes.
///
/// The generated code is representative of real-world HMAT — it mixes
/// keywords, identifiers, numeric and string literals, operators, and
/// significant whitespace, which exercises all major lexer paths.
fn synthetic_source(target_bytes: usize) -> String {
    // One template "line group" is ~140 bytes.
    let chunk = r#"fn compute(x: int, y: float) -> Result<float, str>:
    let result = x * 2 + 3
    let s = "hello world with escapes \n\t\x41"
    let f = 3.14159e2
    if result > 100:
        return Ok(result as float + y)
    return Err("overflow detected")

"#;
    let chunk_len = chunk.len();
    let repeats = (target_bytes / chunk_len).max(1);
    chunk.repeat(repeats)
}

fn bench_lexer_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer_throughput");

    for size_kb in [10u64, 100, 500, 1000] {
        let src = synthetic_source((size_kb * 1024) as usize);
        let bytes = src.len() as u64;
        group.throughput(Throughput::Bytes(bytes));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{size_kb}KB")),
            &src,
            |b, s| {
                b.iter(|| {
                    tokenize(s).expect("synthetic source must lex cleanly");
                });
            },
        );
    }

    group.finish();
}

fn bench_lexer_real_files(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer_real_files");

    let core_lang = include_str!("../../examples/core_language.hm");
    group.throughput(Throughput::Bytes(core_lang.len() as u64));
    group.bench_function("core_language.hm", |b| {
        b.iter(|| tokenize(core_lang).expect("should lex"));
    });

    let ai_chat = include_str!("../../examples/ai_chat.hm");
    group.throughput(Throughput::Bytes(ai_chat.len() as u64));
    group.bench_function("ai_chat.hm", |b| {
        b.iter(|| tokenize(ai_chat).expect("should lex"));
    });

    group.finish();
}

fn bench_lexer_10k_lines(c: &mut Criterion) {
    // Explicit 10 000-line target from the TODO spec.
    let src = synthetic_source(10_000 * 45); // ~45 bytes/line average
    let actual_lines = src.lines().count();
    let bytes = src.len() as u64;

    let mut group = c.benchmark_group("lexer_10k_lines");
    group.throughput(Throughput::Bytes(bytes));
    group.bench_function(format!("{actual_lines}_lines"), |b| {
        b.iter(|| tokenize(&src).expect("should lex"));
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_lexer_throughput,
    bench_lexer_real_files,
    bench_lexer_10k_lines,
);
criterion_main!(benches);
