use std::time::Instant;

use crossy::grid_scandi;
use crossy::grid::Grid;

use std::rc::Rc;
use std::cell::RefCell;

use crossy::{build_crossword_grid_for_command_line, grid_scandi::ScandiGrid};
use rand;
use rand::rand_core::SeedableRng;
use chacha20::ChaCha8Rng;

static WORDS: &str = include_str!("../word_files/open-english-wordnet-dataset-deduped-clean-filtered-by-merged-aspell.tsv");

fn main() {
    let n = 100;
    let mut times: Vec<f64> = Vec::with_capacity(n);
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    let t = Instant::now();
    let d = Rc::new(RefCell::new(grid_scandi::dictionary::Dictionary::build(WORDS)));
    println!("dictionary build: {:.1}ms", t.elapsed().as_secs_f64() * 1000.0);
    std::hint::black_box(&d);

    let mut init_times = Vec::new();
    for _ in 0..20 
    {
        let t = Instant::now();
        let g = ScandiGrid::initialize(14, 14, Rc::clone(&d));
        init_times.push(t.elapsed().as_secs_f64() * 1000.0);
        std::hint::black_box(&g);
    }
    init_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!("initialize median: {:.2}ms", init_times[10]);

    for _ in 0..n 
    {
        let t = Instant::now();
        let grid = build_crossword_grid_for_command_line::<ScandiGrid>(14, 14, &mut rng);
        times.push(t.elapsed().as_secs_f64() * 1000.0);
        std::hint::black_box(&grid);
    }

    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let pct = |p: f64| times[((times.len() as f64 - 1.0) * p) as usize];
    println!("n={}  min={:.1}  p50={:.1}  p90={:.1}  p95={:.1}  p99={:.1}  max={:.1} (ms)",
             times.len(), times[0], pct(0.5), pct(0.9), pct(0.95), pct(0.99), *times.last().unwrap());
}