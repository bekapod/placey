use criterion::{black_box, criterion_group, criterion_main, Criterion};
use placey::placeholder::{generate, GenerateOptions};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("generate 50x50", |b| {
        b.iter(|| {
            generate(black_box(GenerateOptions {
                width: 50,
                ..Default::default()
            }))
        })
    });
    c.bench_function("generate 500x500", |b| {
        b.iter(|| {
            generate(GenerateOptions {
                width: 50,
                ..Default::default()
            })
        })
    });
    c.bench_function("generate 1000x1000", |b| {
        b.iter(|| {
            generate(GenerateOptions {
                width: 1000,
                ..Default::default()
            })
        })
    });
    c.bench_function("generate 100x2500", |b| {
        b.iter(|| {
            generate(GenerateOptions {
                width: 100,
                height: Some(2500),
                ..Default::default()
            })
        })
    });
    c.bench_function("generate 2500x100", |b| {
        b.iter(|| {
            generate(GenerateOptions {
                width: 2500,
                height: Some(100),
                ..Default::default()
            })
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
