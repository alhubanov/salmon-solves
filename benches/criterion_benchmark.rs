use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use crossy::{build_crossword_grid_for_command_line, grid_scandi::ScandiGrid};
use rand;
use rand::rand_core::SeedableRng;
use chacha20::ChaCha8Rng;

fn criterion_benchmark(c: &mut Criterion) 
{
    let mut group = c.benchmark_group("sample_size");
    group.sample_size(300);
    group.noise_threshold(0.20);
    group.significance_level(0.02);
    group.bench_function("grid 5x5", |b| b.iter_batched(
        || ChaCha8Rng::seed_from_u64(2),
        |mut rng| build_crossword_grid_for_command_line::<ScandiGrid>(black_box(5), black_box(5), &mut rng),
        criterion::BatchSize::SmallInput,
    ));
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);