use super::*;
use rand;
use rand::rand_core::SeedableRng;
use chacha20::ChaCha8Rng;

#[test]
fn layout_first_row_and_column() {
    let mut grid: ScandiGrid = ScandiGrid::initialize(5, 5);
    let mut rng = rand::rng();
    
    for _ in 0..10 
    {
        if let Ok(_) = grid.construct(&mut rng) {
            break;
        }
    }

    test_helpers::assert_correct_first_row(&grid);
    test_helpers::assert_correct_first_col(&grid);
}

#[test]
fn construct_is_deterministic_if_rng_is_seeded() 
{
    let grid1 = 
    {
        let mut rng = ChaCha8Rng::seed_from_u64(2);
        let mut grid1 = ScandiGrid::initialize(5, 5);
        grid1.construct(&mut rng).ok();
        grid1.layout
    };

    let grid2 = 
    {
        let mut rng = ChaCha8Rng::seed_from_u64(2);
        let mut grid2 = ScandiGrid::initialize(5, 5);
        grid2.construct(&mut rng).ok();
        grid2.layout
    };

    assert_eq!(grid1, grid2);
}

