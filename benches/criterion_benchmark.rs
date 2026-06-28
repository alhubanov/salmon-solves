use criterion::{criterion_group, criterion_main, Criterion, black_box, BenchmarkId};
use crossy::{build_crossword_grid_for_command_line, grid_scandi::ScandiGrid};
use rand;
use rand::rand_core::SeedableRng;
use chacha20::ChaCha8Rng;
use criterion_perf_events::Perf;
use perfcnt::linux::HardwareEventType as Hardware;
use perfcnt::linux::PerfCounterBuilderLinux as Builder;

fn criterion_benchmark(c: &mut Criterion<Perf>) {
    let mut group = c.benchmark_group("grid_5x5_all_seeds");
    group.sample_size(100);
    group.noise_threshold(0.1);
    group.significance_level(0.02);

    for seed in 1u64..=20 
    {
        group.bench_with_input(
            BenchmarkId::new("grid 5x5", seed),
            &seed,
            |b, &seed| 
            {
                b.iter_batched(
                    || ChaCha8Rng::seed_from_u64(seed),
                    |mut rng| { build_crossword_grid_for_command_line::<ScandiGrid>(black_box(5), black_box(5), &mut rng) },
                    criterion::BatchSize::SmallInput
                );
            },
        );
    }

    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default().with_measurement(Perf::new(Builder::from_hardware_event(Hardware::Instructions)));
    targets = criterion_benchmark
);
criterion_main!(benches);