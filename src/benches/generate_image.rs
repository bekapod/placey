use criterion::{black_box, criterion_group, criterion_main, Criterion};
use placey::placeholder;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("generate 50x50", |b| b.iter(|| placeholder::generate(black_box(50), black_box(50))));
    c.bench_function("generate 500x500", |b| b.iter(|| placeholder::generate(black_box(500), black_box(500))));
    c.bench_function("generate 1000x1000", |b| b.iter(|| placeholder::generate(black_box(1000), black_box(1000))));
    c.bench_function("generate 100x2500", |b| b.iter(|| placeholder::generate(black_box(100), black_box(2500))));
    c.bench_function("generate 2500x100", |b| b.iter(|| placeholder::generate(black_box(2500), black_box(100))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);