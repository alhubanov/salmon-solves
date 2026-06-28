use super::*;
use rand;

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

