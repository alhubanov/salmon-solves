use super::*;

#[test]
fn layout_first_row_and_column() {
    let mut grid: ScandiGrid = ScandiGrid::initialize(5, 5);
    grid.construct();

    println!("0");

    test_helpers::assert_correct_first_row(&grid);
    test_helpers::assert_correct_first_col(&grid);
}

