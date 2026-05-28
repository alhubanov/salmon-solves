
use super::*;

pub fn assert_cell_is_filled(cell: &GridCell, letter: char) {
    assert_eq!(cell.cell_value, letter);
    assert_eq!(cell.cell_state, CellState::Filled);
}

pub fn assert_cell_is_not_filled(cell: &GridCell) {
    assert_eq!(cell.cell_value, '_');
    assert_eq!(cell.cell_state, CellState::NotFilled);
}

pub fn assert_horizontal_word_placement_starting_at_idx(grid: &SimpleGrid, height_idx: usize, width_idx: usize, word: &String) {

    let mut count = 0;
    for letter in word.chars() {
        assert_cell_is_filled(&grid.layout[height_idx][width_idx + count], letter);
        count += 1;
    }
}

pub fn assert_vertical_word_placement_starting_at_idx(grid: &SimpleGrid, height_idx: usize, width_idx: usize, word: &String) {

    let mut count = 0;
    for letter in word.chars() {
        assert_cell_is_filled(&grid.layout[height_idx + count][width_idx], letter);
        count += 1;
    }
}