use criterion::{criterion_group, criterion_main, Criterion, black_box, BenchmarkId};
use crossy::{build_crossword_grid_for_command_line, grid_scandi::ScandiGrid};
use rand;
use rand::rand_core::SeedableRng;
use chacha20::ChaCha8Rng;
use criterion_perf_events::Perf;
use perfcnt::linux::HardwareEventType as Hardware;
use perfcnt::linux::PerfCounterBuilderLinux as Builder;

// Before running: 
// 1. Disable turbo boost - echo 1 | sudo tee /sys/devices/system/cpu/intel_pstate/no_turbo
// 2. Pin CPU frequency - sudo cpupower frequency-set -g performance
//
// To run pinned to a single core:
// taskset -c 0 cargo bench
//
// In case of a 'permission denied' error: sudo sh -c 'echo 1 >/proc/sys/kernel/perf_event_paranoid'
fn criterion_benchmark(c: &mut Criterion<Perf>) 
{
    let mut group = c.benchmark_group("grid_8x8_all_seeds");
    group.sample_size(100);
    group.noise_threshold(0.05);
    group.significance_level(0.02);

    for seed in 1u64..=20 
    {
        group.bench_with_input
        (
            BenchmarkId::new("grid 8x8", seed),
            &seed,
            |b, &seed| 
            {
                b.iter_batched
                (
                    || ChaCha8Rng::seed_from_u64(seed),
                    |mut rng| { build_crossword_grid_for_command_line::<ScandiGrid>(black_box(8), black_box(8), &mut rng) },
                    criterion::BatchSize::PerIteration
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