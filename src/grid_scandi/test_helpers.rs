use super::*;

#[allow(dead_code)]
fn is_clue(cell: &GridCell) -> bool {
    cell.assigned_cell_states.iter().any(|state| matches!(state, PossibleCellState::Clue(_)))
}

#[allow(dead_code)]
fn is_letter(cell: &GridCell) -> bool {
    cell.assigned_cell_states.iter().any(|state| matches!(state, PossibleCellState::Letter))
}

#[allow(dead_code)]
fn slot_is_to_the_right_and_vertical(cell: &GridCell) -> bool {
    cell.assigned_cell_states.iter().any(|state| matches!(state, PossibleCellState::Clue(SlotDirection::DownOnRightSide)))
}

#[allow(dead_code)]
fn slot_is_below_and_horizontal(cell: &GridCell) -> bool {
    cell.assigned_cell_states.iter().any(|state| matches!(state, PossibleCellState::Clue(SlotDirection::RightOnBottomSide)))
}

#[allow(dead_code)]
fn slot_is_to_the_right_and_horizontal(cell: &GridCell) -> bool {
    cell.assigned_cell_states.iter().any(|state| matches!(state, PossibleCellState::Clue(SlotDirection::Right)))
}

#[allow(dead_code)]
fn slot_is_below_and_vertical(cell: &GridCell) -> bool {
    cell.assigned_cell_states.iter().any(|state| matches!(state, PossibleCellState::Clue(SlotDirection::Down)))
}

#[allow(dead_code)]
fn assert_correct_top_left_cell(grid: &ScandiGrid) {
    assert!(is_clue(&grid.layout[0][0].borrow()));
    assert!(!is_letter(&grid.layout[0][0].borrow()));

    assert!(slot_is_below_and_horizontal(&grid.layout[0][0].borrow()) || slot_is_to_the_right_and_vertical(&grid.layout[0][0].borrow()));
    assert!(!slot_is_to_the_right_and_horizontal(&grid.layout[0][0].borrow()) && !slot_is_below_and_vertical(&grid.layout[0][0].borrow()));
}

#[allow(dead_code)]
pub fn assert_correct_first_row(grid: &ScandiGrid) {
    assert_correct_top_left_cell(grid);

    for idx in 1..grid.width {
        
        if is_letter(&grid.layout[0][idx as usize].borrow()) {
            assert!(slot_is_to_the_right_and_vertical(&grid.layout[0][(idx - 1) as usize].borrow()));
            assert!(!slot_is_to_the_right_and_horizontal(&grid.layout[0][(idx - 1) as usize].borrow()));

            if idx != 1 {
                assert!(!slot_is_below_and_horizontal(&grid.layout[0][(idx - 1) as usize].borrow()));
            }
        }

        if is_clue(&grid.layout[0][idx as usize].borrow()) {
            assert!(!slot_is_to_the_right_and_vertical(&grid.layout[0][(idx - 1) as usize].borrow()));
            assert!(!slot_is_to_the_right_and_horizontal(&grid.layout[0][(idx - 1) as usize].borrow()));
            assert!(!slot_is_below_and_horizontal(&grid.layout[0][(idx - 1) as usize].borrow()));

            assert!(!slot_is_to_the_right_and_horizontal(&grid.layout[0][idx as usize].borrow()));
            assert!(!slot_is_below_and_horizontal(&grid.layout[0][idx as usize].borrow()));
        }
    }
}

#[allow(dead_code)]
pub fn assert_correct_first_col(grid: &ScandiGrid) {
    assert_correct_top_left_cell(grid);

    for idx in 1..grid.height {
        
        if is_letter(&grid.layout[idx as usize][0].borrow()) {
            assert!(!slot_is_below_and_vertical(&grid.layout[(idx - 1) as usize][0].borrow()));
            assert!(slot_is_below_and_horizontal(&grid.layout[(idx - 1) as usize][0].borrow()));

            if idx != 1 {
                assert!(!slot_is_to_the_right_and_vertical(&grid.layout[(idx - 1) as usize][0].borrow()));
            }
        }

        if is_clue(&grid.layout[idx as usize][0].borrow()) {
            assert!(!slot_is_below_and_horizontal(&grid.layout[(idx - 1) as usize][0].borrow()));
            assert!(!slot_is_to_the_right_and_vertical(&grid.layout[(idx - 1) as usize][0].borrow()));
            assert!(!slot_is_below_and_vertical(&grid.layout[(idx - 1) as usize][0].borrow()));

            assert!(!slot_is_to_the_right_and_vertical(&grid.layout[idx as usize][0].borrow()));
            assert!(!slot_is_below_and_vertical(&grid.layout[idx as usize][0].borrow()));
        }
    }
}