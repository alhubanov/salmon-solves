use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use crossy::{build_crossword_grid_for_command_line, grid_scandi::ScandiGrid};
use rand;

fn criterion_benchmark(c: &mut Criterion) 
{
    let mut rng = rand::rng();

    let mut group = c.benchmark_group("sample_size");
    group.sample_size(200);
    group.bench_function("grid 5x5", |b| b.iter(|| build_crossword_grid_for_command_line::<ScandiGrid>(black_box(5), black_box(5), black_box(&mut rng))));
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);