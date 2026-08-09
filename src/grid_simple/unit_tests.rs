use super::*;
use super::test_helpers;
use std::rc::Rc;
use std::cell::RefCell;
use crate::grid_scandi::dictionary;

static WORDS: &str = include_str!("../../word_files/oewn-answer-clue-deduped-clean.tsv");

#[test]
fn cell_reset() {

    let dictionary = Rc::new(RefCell::new(dictionary::Dictionary::build(WORDS)));

    let mut grid = SimpleGrid::initialize(1, 1, dictionary);

    // case Filled
    grid.layout[0 as usize][0 as usize].cell_value = 'a';
    grid.layout[0 as usize][0 as usize].cell_state = CellState::Filled;

    grid.reset_cell_at_index(0, 0);
    test_helpers::assert_cell_is_filled(&grid.layout[0 as usize][0 as usize], 'a');

    // case TentativelyFilled
    grid.layout[0 as usize][0 as usize].cell_value = '1';
    grid.layout[0 as usize][0 as usize].cell_state = CellState::TentativelyFilled;

    grid.reset_cell_at_index(0, 0);
    test_helpers::assert_cell_is_not_filled(&grid.layout[0 as usize][0 as usize]);

    // case Empty
    grid.layout[0 as usize][0 as usize].cell_value = '_';
    grid.layout[0 as usize][0 as usize].cell_state = CellState::NotFilled;

    grid.reset_cell_at_index(0, 0);
    test_helpers::assert_cell_is_not_filled(&grid.layout[0 as usize][0 as usize]);
}

#[test]
fn cell_confirmation() {

    let dictionary = Rc::new(RefCell::new(dictionary::Dictionary::build(WORDS)));
    let mut grid = SimpleGrid::initialize(1, 1, dictionary);

    // case Filled
    grid.layout[0 as usize][0 as usize].cell_value = 'a';
    grid.layout[0 as usize][0 as usize].cell_state = CellState::Filled;

    grid.confirm_cell_at_index(0, 0);
    test_helpers::assert_cell_is_filled(&grid.layout[0 as usize][0 as usize], 'a');

    // case TentativelyFilled
    grid.layout[0 as usize][0 as usize].cell_value = '1';
    grid.layout[0 as usize][0 as usize].cell_state = CellState::TentativelyFilled;

    grid.confirm_cell_at_index(0, 0);
    test_helpers::assert_cell_is_filled(&grid.layout[0 as usize][0 as usize], '1');

    // case Empty
    grid.layout[0 as usize][0 as usize].cell_value = 'q';
    grid.layout[0 as usize][0 as usize].cell_state = CellState::NotFilled;

    grid.confirm_cell_at_index(0, 0);
    test_helpers::assert_cell_is_filled(&grid.layout[0 as usize][0 as usize], 'q');
}

#[test]
fn placing_words_horizontally() {

    let dictionary = Rc::new(RefCell::new(dictionary::Dictionary::build(WORDS)));
    let mut grid = SimpleGrid::initialize(4, 4, dictionary);
    let result = grid.place_word_horizontally_starting_at_given_coordinates(0, 0, &"word".to_owned());

    assert!(result.is_ok());
    test_helpers::assert_horizontal_word_placement_starting_at_idx(&grid, 0, 0, &"word".to_owned());

    let result = grid.place_word_horizontally_starting_at_given_coordinates(0, 1, &"the".to_owned());

    assert!(result.is_err());
    test_helpers::assert_horizontal_word_placement_starting_at_idx(&grid, 0, 0, &"word".to_owned());

    let result = grid.place_word_horizontally_starting_at_given_coordinates(0, 0, &"moat".to_owned());

    assert!(result.is_err());
    test_helpers::assert_horizontal_word_placement_starting_at_idx(&grid, 0, 0, &"word".to_owned());
}

#[test]
fn placing_words_horizontally_with_crossing_at_start() {

    let dictionary = Rc::new(RefCell::new(dictionary::Dictionary::build(WORDS)));
    let mut grid = SimpleGrid::initialize(5, 5, dictionary);
    grid.layout[0 as usize][0 as usize].cell_value = 'm';
    grid.layout[0 as usize][0 as usize].cell_state = CellState::Filled;
    grid.layout[1 as usize][0 as usize].cell_value = 'a';
    grid.layout[1 as usize][0 as usize].cell_state = CellState::Filled;
    grid.layout[2 as usize][0 as usize].cell_value = 'p';
    grid.layout[2 as usize][0 as usize].cell_state = CellState::Filled;
    grid.layout[3 as usize][0 as usize].cell_value = 'l';
    grid.layout[3 as usize][0 as usize].cell_state = CellState::Filled;
    grid.layout[4 as usize][0 as usize].cell_value = 'e';
    grid.layout[4 as usize][0 as usize].cell_state = CellState::Filled;

    let result = grid.place_word_horizontally_starting_at_given_coordinates(0, 0, &"toad".to_owned());
    assert!(result.is_err());

    let result = grid.place_word_horizontally_starting_at_given_coordinates(0, 0, &"moat".to_owned());
    assert!(result.is_ok());
    test_helpers::assert_horizontal_word_placement_starting_at_idx(&grid, 0, 0, &"moat".to_owned());
    test_helpers::assert_vertical_word_placement_starting_at_idx(&grid, 0, 0, &"maple".to_owned());

    let result = grid.place_word_horizontally_starting_at_given_coordinates(2, 0, &"star".to_owned());
    assert!(result.is_err());

    let result = grid.place_word_horizontally_starting_at_given_coordinates(2, 0, &"post".to_owned());
    assert!(result.is_ok());
    test_helpers::assert_horizontal_word_placement_starting_at_idx(&grid, 2, 0, &"post".to_owned());
    test_helpers::assert_vertical_word_placement_starting_at_idx(&grid, 0, 0, &"maple".to_owned());

    let result = grid.place_word_horizontally_starting_at_given_coordinates(4, 0, &"star".to_owned());
    assert!(result.is_err());

    let result = grid.place_word_horizontally_starting_at_given_coordinates(4, 0, &"east".to_owned());
    assert!(result.is_ok());
    test_helpers::assert_horizontal_word_placement_starting_at_idx(&grid, 4, 0, &"east".to_owned());
    test_helpers::assert_vertical_word_placement_starting_at_idx(&grid, 0, 0, &"maple".to_owned());

    assert!(grid.placed_words.len() == 3);
}

#[test]
fn placing_words_horizontally_with_crossing_in_the_middle() {

    let dictionary = Rc::new(RefCell::new(dictionary::Dictionary::build(WORDS)));
    let mut grid = SimpleGrid::initialize(5, 5, dictionary);
    grid.layout[0 as usize][2 as usize].cell_value = 'm';
    grid.layout[0 as usize][2 as usize].cell_state = CellState::Filled;
    grid.layout[1 as usize][2 as usize].cell_value = 'a';
    grid.layout[1 as usize][2 as usize].cell_state = CellState::Filled;
    grid.layout[2 as usize][2 as usize].cell_value = 'p';
    grid.layout[2 as usize][2 as usize].cell_state = CellState::Filled;
    grid.layout[3 as usize][2 as usize].cell_value = 'l';
    grid.layout[3 as usize][2 as usize].cell_state = CellState::Filled;
    grid.layout[4 as usize][2 as usize].cell_value = 'e';
    grid.layout[4 as usize][2 as usize].cell_state = CellState::Filled;

    let result = grid.place_word_horizontally_starting_at_given_coordinates(0, 0, &"toad".to_owned());
    assert!(result.is_err());

    let result = grid.place_word_horizontally_starting_at_given_coordinates(0, 0, &"momma".to_owned());
    assert!(result.is_ok());
    test_helpers::assert_horizontal_word_placement_starting_at_idx(&grid, 0, 0, &"momma".to_owned());
    test_helpers::assert_vertical_word_placement_starting_at_idx(&grid, 0, 2, &"maple".to_owned());

    let result = grid.place_word_horizontally_starting_at_given_coordinates(2, 0, &"star".to_owned());
    assert!(result.is_err());

    let result = grid.place_word_horizontally_starting_at_given_coordinates(2, 0, &"cops".to_owned());
    assert!(result.is_ok());
    test_helpers::assert_horizontal_word_placement_starting_at_idx(&grid, 2, 0, &"cops".to_owned());
    test_helpers::assert_vertical_word_placement_starting_at_idx(&grid, 0, 2, &"maple".to_owned());

    let result = grid.place_word_horizontally_starting_at_given_coordinates(4, 0, &"star".to_owned());
    assert!(result.is_err());

    let result = grid.place_word_horizontally_starting_at_given_coordinates(4, 0, &"fred".to_owned());
    assert!(result.is_ok());
    test_helpers::assert_horizontal_word_placement_starting_at_idx(&grid, 4, 0, &"fred".to_owned());
    test_helpers::assert_vertical_word_placement_starting_at_idx(&grid, 0, 2, &"maple".to_owned());

    assert!(grid.placed_words.len() == 3);
}

#[test]
fn placing_words_horizontally_with_crossing_in_the_end() {

    let dictionary = Rc::new(RefCell::new(dictionary::Dictionary::build(WORDS)));
    let mut grid = SimpleGrid::initialize(5, 5, dictionary);
    grid.layout[0 as usize][4 as usize].cell_value = 'm';
    grid.layout[0 as usize][4 as usize].cell_state = CellState::Filled;
    grid.layout[1 as usize][4 as usize].cell_value = 'a';
    grid.layout[1 as usize][4 as usize].cell_state = CellState::Filled;
    grid.layout[2 as usize][4 as usize].cell_value = 'p';
    grid.layout[2 as usize][4 as usize].cell_state = CellState::Filled;
    grid.layout[3 as usize][4 as usize].cell_value = 'l';
    grid.layout[3 as usize][4 as usize].cell_state = CellState::Filled;
    grid.layout[4 as usize][4 as usize].cell_value = 'e';
    grid.layout[4 as usize][4 as usize].cell_state = CellState::Filled;

    let result = grid.place_word_horizontally_starting_at_given_coordinates(0, 0, &"toad".to_owned());
    assert!(result.is_err());

    let result = grid.place_word_horizontally_starting_at_given_coordinates(0, 0, &"bitum".to_owned());
    assert!(result.is_ok());
    test_helpers::assert_horizontal_word_placement_starting_at_idx(&grid, 0, 0, &"bitum".to_owned());
    test_helpers::assert_vertical_word_placement_starting_at_idx(&grid, 0, 4, &"maple".to_owned());

    let result = grid.place_word_horizontally_starting_at_given_coordinates(2, 0, &"start".to_owned());
    assert!(result.is_err());

    let result = grid.place_word_horizontally_starting_at_given_coordinates(2, 0, &"aesop".to_owned());
    assert!(result.is_ok());
    test_helpers::assert_horizontal_word_placement_starting_at_idx(&grid, 2, 0, &"aesop".to_owned());
    test_helpers::assert_vertical_word_placement_starting_at_idx(&grid, 0, 4, &"maple".to_owned());

    let result = grid.place_word_horizontally_starting_at_given_coordinates(4, 0, &"start".to_owned());
    assert!(result.is_err());

    let result = grid.place_word_horizontally_starting_at_given_coordinates(4, 0, &"andre".to_owned());
    assert!(result.is_ok());
    test_helpers::assert_horizontal_word_placement_starting_at_idx(&grid, 4, 0, &"andre".to_owned());
    test_helpers::assert_vertical_word_placement_starting_at_idx(&grid, 0, 4, &"maple".to_owned());

    assert!(grid.placed_words.len() == 3);
}

#[test]
fn placing_words_vertically() {

    let dictionary = Rc::new(RefCell::new(dictionary::Dictionary::build(WORDS)));
    let mut grid = SimpleGrid::initialize(4, 4, dictionary);
    let result = grid.place_word_vertically_starting_at_given_coordinates(0, 0, &"word".to_owned());

    assert!(result.is_ok());
    test_helpers::assert_vertical_word_placement_starting_at_idx(&grid, 0, 0, &"word".to_owned());

    let result = grid.place_word_vertically_starting_at_given_coordinates(1, 0, &"the".to_owned());

    assert!(result.is_err());
    test_helpers::assert_vertical_word_placement_starting_at_idx(&grid, 0, 0, &"word".to_owned());

    let result = grid.place_word_vertically_starting_at_given_coordinates(0, 0, &"moat".to_owned());

    assert!(result.is_err());
    test_helpers::assert_vertical_word_placement_starting_at_idx(&grid, 0, 0, &"word".to_owned());
}

#[test]
fn placing_words_vertically_with_crossing_at_start() {

    let dictionary = Rc::new(RefCell::new(dictionary::Dictionary::build(WORDS)));
    let mut grid = SimpleGrid::initialize(5, 5, dictionary);
    grid.layout[0 as usize][0 as usize].cell_value = 'm';
    grid.layout[0 as usize][0 as usize].cell_state = CellState::Filled;
    grid.layout[0 as usize][1 as usize].cell_value = 'a';
    grid.layout[0 as usize][1 as usize].cell_state = CellState::Filled;
    grid.layout[0 as usize][2 as usize].cell_value = 'p';
    grid.layout[0 as usize][2 as usize].cell_state = CellState::Filled;
    grid.layout[0 as usize][3 as usize].cell_value = 'l';
    grid.layout[0 as usize][3 as usize].cell_state = CellState::Filled;
    grid.layout[0 as usize][4 as usize].cell_value = 'e';
    grid.layout[0 as usize][4 as usize].cell_state = CellState::Filled;

    let result = grid.place_word_vertically_starting_at_given_coordinates(0, 0, &"toad".to_owned());
    assert!(result.is_err());

    let result = grid.place_word_vertically_starting_at_given_coordinates(0, 0, &"moat".to_owned());
    assert!(result.is_ok());
    test_helpers::assert_horizontal_word_placement_starting_at_idx(&grid, 0, 0, &"maple".to_owned());
    test_helpers::assert_vertical_word_placement_starting_at_idx(&grid, 0, 0, &"moat".to_owned());

    let result = grid.place_word_vertically_starting_at_given_coordinates(0, 2, &"star".to_owned());
    assert!(result.is_err());

    let result = grid.place_word_vertically_starting_at_given_coordinates(0, 2, &"post".to_owned());
    assert!(result.is_ok());
    test_helpers::assert_horizontal_word_placement_starting_at_idx(&grid, 0, 0, &"maple".to_owned());
    test_helpers::assert_vertical_word_placement_starting_at_idx(&grid, 0, 2, &"post".to_owned());

    let result = grid.place_word_vertically_starting_at_given_coordinates(0, 4, &"star".to_owned());
    assert!(result.is_err());

    let result = grid.place_word_vertically_starting_at_given_coordinates(0, 4, &"east".to_owned());
    assert!(result.is_ok());
    test_helpers::assert_horizontal_word_placement_starting_at_idx(&grid, 0, 0, &"maple".to_owned());
    test_helpers::assert_vertical_word_placement_starting_at_idx(&grid, 0, 4, &"east".to_owned());

    assert!(grid.placed_words.len() == 3);
}

#[test]
fn placing_words_vertically_with_crossing_in_the_middle() {

    let dictionary = Rc::new(RefCell::new(dictionary::Dictionary::build(WORDS)));
    let mut grid = SimpleGrid::initialize(5, 5, dictionary);
    grid.layout[2 as usize][0 as usize].cell_value = 'm';
    grid.layout[2 as usize][0 as usize].cell_state = CellState::Filled;
    grid.layout[2 as usize][1 as usize].cell_value = 'a';
    grid.layout[2 as usize][1 as usize].cell_state = CellState::Filled;
    grid.layout[2 as usize][2 as usize].cell_value = 'p';
    grid.layout[2 as usize][2 as usize].cell_state = CellState::Filled;
    grid.layout[2 as usize][3 as usize].cell_value = 'l';
    grid.layout[2 as usize][3 as usize].cell_state = CellState::Filled;
    grid.layout[2 as usize][4 as usize].cell_value = 'e';
    grid.layout[2 as usize][4 as usize].cell_state = CellState::Filled;

    let result = grid.place_word_vertically_starting_at_given_coordinates(0, 0, &"toad".to_owned());
    assert!(result.is_err());

    let result = grid.place_word_vertically_starting_at_given_coordinates(0, 0, &"momma".to_owned());
    assert!(result.is_ok());
    test_helpers::assert_horizontal_word_placement_starting_at_idx(&grid, 2, 0, &"maple".to_owned());
    test_helpers::assert_vertical_word_placement_starting_at_idx(&grid, 0, 0, &"momma".to_owned());

    let result = grid.place_word_vertically_starting_at_given_coordinates(0, 2, &"star".to_owned());
    assert!(result.is_err());

    let result = grid.place_word_vertically_starting_at_given_coordinates(0, 2, &"cops".to_owned());
    assert!(result.is_ok());
    test_helpers::assert_horizontal_word_placement_starting_at_idx(&grid, 2, 0, &"maple".to_owned());
    test_helpers::assert_vertical_word_placement_starting_at_idx(&grid, 0, 2, &"cops".to_owned());

    let result = grid.place_word_vertically_starting_at_given_coordinates(0, 4, &"star".to_owned());
    assert!(result.is_err());

    let result = grid.place_word_vertically_starting_at_given_coordinates(0, 4, &"fred".to_owned());
    assert!(result.is_ok());
    test_helpers::assert_horizontal_word_placement_starting_at_idx(&grid, 2, 0, &"maple".to_owned());
    test_helpers::assert_vertical_word_placement_starting_at_idx(&grid, 0, 4, &"fred".to_owned());

    assert!(grid.placed_words.len() == 3);
}

#[test]
fn placing_words_vertically_with_crossing_in_the_end() {

    let dictionary = Rc::new(RefCell::new(dictionary::Dictionary::build(WORDS)));
    let mut grid = SimpleGrid::initialize(5, 5, dictionary);
    grid.layout[4 as usize][0 as usize].cell_value = 'm';
    grid.layout[4 as usize][0 as usize].cell_state = CellState::Filled;
    grid.layout[4 as usize][1 as usize].cell_value = 'a';
    grid.layout[4 as usize][1 as usize].cell_state = CellState::Filled;
    grid.layout[4 as usize][2 as usize].cell_value = 'p';
    grid.layout[4 as usize][2 as usize].cell_state = CellState::Filled;
    grid.layout[4 as usize][3 as usize].cell_value = 'l';
    grid.layout[4 as usize][3 as usize].cell_state = CellState::Filled;
    grid.layout[4 as usize][4 as usize].cell_value = 'e';
    grid.layout[4 as usize][4 as usize].cell_state = CellState::Filled;

    let result = grid.place_word_vertically_starting_at_given_coordinates(0, 0, &"toad".to_owned());
    assert!(result.is_err());

    let result = grid.place_word_vertically_starting_at_given_coordinates(0, 0, &"bitum".to_owned());
    assert!(result.is_ok());
    test_helpers::assert_horizontal_word_placement_starting_at_idx(&grid, 4, 0, &"maple".to_owned());
    test_helpers::assert_vertical_word_placement_starting_at_idx(&grid, 0, 0, &"bitum".to_owned());

    let result = grid.place_word_vertically_starting_at_given_coordinates(0, 2, &"start".to_owned());
    assert!(result.is_err());

    let result = grid.place_word_vertically_starting_at_given_coordinates(0, 2, &"aesop".to_owned());
    assert!(result.is_ok());
    test_helpers::assert_horizontal_word_placement_starting_at_idx(&grid, 4, 0, &"maple".to_owned());
    test_helpers::assert_vertical_word_placement_starting_at_idx(&grid, 0, 2, &"aesop".to_owned());

    let result = grid.place_word_vertically_starting_at_given_coordinates(0, 4, &"start".to_owned());
    assert!(result.is_err());

    let result = grid.place_word_vertically_starting_at_given_coordinates(0, 4, &"andre".to_owned());
    assert!(result.is_ok());
    test_helpers::assert_horizontal_word_placement_starting_at_idx(&grid, 4, 0, &"maple".to_owned());
    test_helpers::assert_vertical_word_placement_starting_at_idx(&grid, 0, 4, &"andre".to_owned());

    assert!(grid.placed_words.len() == 3);
}

#[test]
fn placing_words_out_of_bounds() {

    let dictionary = Rc::new(RefCell::new(dictionary::Dictionary::build(WORDS)));
    let mut grid = SimpleGrid::initialize(5, 5, dictionary);

    let result = grid.place_word_horizontally_starting_at_given_coordinates(4, 0, &"map".to_owned());
    assert!(result.is_ok());

    let result = grid.place_word_vertically_starting_at_given_coordinates(0, 4, &"curtain".to_owned());
    assert!(result.is_err());

    let result = grid.place_word_horizontally_starting_at_given_coordinates(0, 0, &"program".to_owned());
    assert!(result.is_err());

    assert!(grid.placed_words.len() == 1);
}

#[test]
fn placing_words_too_close_1() {

    let dictionary = Rc::new(RefCell::new(dictionary::Dictionary::build(WORDS)));
    let mut grid = SimpleGrid::initialize(7, 7, dictionary);

    let result = grid.place_word_horizontally_starting_at_given_coordinates(2, 0, &"map".to_owned());
    assert!(result.is_ok());

    let result = grid.place_word_horizontally_starting_at_given_coordinates(2, 3, &"the".to_owned());
    assert!(result.is_err());

    let result = grid.place_word_vertically_starting_at_given_coordinates(3, 1, &"the".to_owned());
    assert!(result.is_err());

    let result = grid.place_word_horizontally_starting_at_given_coordinates(3, 1, &"debt".to_owned());
    assert!(result.is_err());

    let result = grid.place_word_horizontally_starting_at_given_coordinates(1, 1, &"baby".to_owned());
    assert!(result.is_err());

    assert!(grid.placed_words.len() == 1);
}

#[test]
fn placing_words_too_close_2() {

    let dictionary = Rc::new(RefCell::new(dictionary::Dictionary::build(WORDS)));
    let mut grid = SimpleGrid::initialize(7, 7, dictionary);

    let result = grid.place_word_vertically_starting_at_given_coordinates(0, 2, &"map".to_owned());
    assert!(result.is_ok());

    let result = grid.place_word_vertically_starting_at_given_coordinates(3, 2, &"the".to_owned());
    assert!(result.is_err());

    let result = grid.place_word_horizontally_starting_at_given_coordinates(1, 3, &"the".to_owned());
    assert!(result.is_err());

    let result = grid.place_word_vertically_starting_at_given_coordinates(2, 1, &"dad".to_owned());
    assert!(result.is_err());

    let result = grid.place_word_vertically_starting_at_given_coordinates(2, 3, &"bee".to_owned());
    assert!(result.is_err());

    assert!(grid.placed_words.len() == 1);
}

#[test]
fn impossible_crossing_1() {

    let dictionary = Rc::new(RefCell::new(dictionary::Dictionary::build(WORDS)));
    let mut grid = SimpleGrid::initialize(5, 5, dictionary);

    let result = grid.place_word_horizontally_starting_at_given_coordinates(2, 0, &"maple".to_owned());
    assert!(result.is_ok());

    let result = grid.place_word_vertically_starting_at_given_coordinates(0, 2, &"baste".to_owned());
    assert!(result.is_err());

    assert!(grid.placed_words.len() == 1);
}

#[test]
fn impossible_crossing_2() {

    let dictionary = Rc::new(RefCell::new(dictionary::Dictionary::build(WORDS)));
    let mut grid = SimpleGrid::initialize(5, 5, dictionary);

    let result = grid.place_word_vertically_starting_at_given_coordinates(0, 2, &"baste".to_owned());
    assert!(result.is_ok());

    let result = grid.place_word_horizontally_starting_at_given_coordinates(2, 0, &"maple".to_owned());
    assert!(result.is_err());

    assert!(grid.placed_words.len() == 1);
}

#[test]
fn placing_first_word_1() {

    let dictionary = Rc::new(RefCell::new(dictionary::Dictionary::build(WORDS)));
    let mut grid = SimpleGrid::initialize(5, 5, dictionary);

    let result = grid.place_first_word(&"bass".to_owned());
    assert!(result.is_ok());
    test_helpers::assert_horizontal_word_placement_starting_at_idx(&grid, 2, 1, &"bass".to_owned());
    assert!(grid.placed_words.len() == 1);
}

#[test]
fn placing_first_word_2() {

    let dictionary = Rc::new(RefCell::new(dictionary::Dictionary::build(WORDS)));
    let mut grid = SimpleGrid::initialize(6, 6, dictionary);

    let result = grid.place_first_word(&"bass".to_owned());
    assert!(result.is_ok());
    test_helpers::assert_horizontal_word_placement_starting_at_idx(&grid, 2, 1, &"bass".to_owned());
    assert!(grid.placed_words.len() == 1);
}
