use super::*;

#[test]
fn layout_first_row_and_column() {
    let mut grid: ScandiGrid = ScandiGrid::initialize(5, 5);
    
    for _ in 0..10 {
        if let Ok(_) = grid.construct() {
            break;
        }
    }

    test_helpers::assert_correct_first_row(&grid);
    test_helpers::assert_correct_first_col(&grid);
}

