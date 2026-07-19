use num::Integer;
use rand::{self, seq::SliceRandom};
use rand::rand_core::Rng;

use ahash::AHashSet;

#[cfg(test)]
mod unit_tests;
#[cfg(test)]
mod test_helpers;

mod gridcell;
use gridcell::{GridCell, CellState, CrossingState};
use crate::grid::Grid;
use crate::grid::LayoutError;
use crate::grid_scandi::run_stats::RunStats;

static WORDS: &str = include_str!("../word_files/common_english_words_long.txt");

pub enum WordPlacementError {
    DoesNotFitByWidth,
    DoesNotFitByHeight,
    BlockerCurrent,
    BlockerRight,
    BlockerLeft,
    BlockerTop,
    BlockerBottom,
    NoAvailableSpace,
    WordAlreadyContained
}

#[derive(Eq, Hash, PartialEq)]
enum WordDirection {
    Across,
    Down
}

pub struct SimpleGrid {
    width: u32,
    height: u32,
    layout: Vec<Vec<GridCell>>,
    placed_words: AHashSet<String>
}

impl Grid for SimpleGrid {
    fn initialize(width: u32, height: u32) -> Self {
        let mut layout : Vec<Vec<GridCell>> = Vec::new();

        for row_idx in 0..height {
            layout.push(Vec::new());

            for _ in 0..width {
                let cell = GridCell::new();
                layout[row_idx as usize].push(cell);
            }
        }

        let placed_words = AHashSet::new();

        Self { width, height, layout, placed_words }
    }

    fn construct(&mut self, rng: &mut dyn Rng, _max_depth: u32, _run_stats: &mut RunStats) -> Result<(), LayoutError> {
        let mut words_vec : Vec<&str> = WORDS.lines().collect();
        words_vec.shuffle(rng);

        for word in words_vec {
            if let Err(_) = self.place_word(&word.to_owned()) {
                continue;
            }
        }

        Ok(())
    }

    fn print(&self) -> () {
        for vertical_idx in 0..self.height {
            for horizontal_idx in 0..self.width {
                if self.layout[vertical_idx as usize][horizontal_idx as usize].cell_state == CellState::NotFilled {
                    print!("# ");
                } else {
                    print!{"{} ", self.layout[vertical_idx as usize][horizontal_idx as usize].cell_value}
                }
            }

            println!()
        }
    }
}

impl SimpleGrid {

    fn place_word(&mut self, word: &String) -> Result<(), WordPlacementError> {

        if self.placed_words.is_empty() 
        {
            return self.place_first_word(&word.to_owned());
        } 
        else if !self.placed_words.contains(word) 
        {
            return self.attempt_to_place_word(&word.to_owned());
        }

        Err(WordPlacementError::WordAlreadyContained)
    }

    fn attempt_to_place_word(&mut self, word: &String) -> Result<(), WordPlacementError> {

        for horizontal_idx in 0..self.width {
            for vertical_idx in 0..self.height {
                if let Ok(_) = self.place_word_horizontally_starting_at_given_coordinates(vertical_idx, horizontal_idx, word) {
                    return Ok(());
                }

                if let Ok(_) = self.place_word_vertically_starting_at_given_coordinates(vertical_idx, horizontal_idx, word) {
                    return Ok(());
                }
            }
        }

        Err(WordPlacementError::NoAvailableSpace)
    }

    fn place_first_word(&mut self, word: &String) -> Result<(), WordPlacementError>{

        let middle_row_idx = match self.height.is_even() {
            true => self.height / 2 - 1,
            false => self.height / 2
        };
        
        self.place_word_horizontally_starting_at_given_coordinates(middle_row_idx, 1, word)?;

        Ok(())
    }

    fn apply_on_sequence<F>(
        &mut self, starting_coord: u32, mut height: u32, mut width: u32, direction: WordDirection, mut operation: F,
    )
    where
        F: FnMut(&mut Self, u32, u32),
    {
        match direction {
            WordDirection::Across => {
                while width >= starting_coord {
                    operation(self, height, width);
                    if width == 0 {
                        break;
                    }
                    width -= 1;
                }
            },
            WordDirection::Down => {
                while height >= starting_coord {
                    operation(self, height, width);
                    if height == 0 {
                        break;
                    }
                    height -= 1;
                }
            }
        }
    }

    fn clear_word(&mut self, starting_coord: u32, height_coord: u32, width_coord: u32, direction: WordDirection) {
        self.apply_on_sequence(starting_coord, height_coord, width_coord, direction, |grid, h, w| {
            grid.reset_cell_at_index(h, w)
        });
    }

    fn confirm_word(&mut self, starting_coord: u32, height_coord: u32, width_coord: u32, direction: WordDirection) {
       self.apply_on_sequence(starting_coord, height_coord, width_coord, direction, |grid, h, w| {
            grid.confirm_cell_at_index(h, w)
        });
    }

    fn place_word_horizontally_starting_at_given_coordinates(&mut self, height_coord: u32, mut width_coord: u32, word: &String) -> Result<(), WordPlacementError> {

        if word.len() as u32 > self.width - width_coord {
            return Err(WordPlacementError::DoesNotFitByWidth);
        }

        let starting_width_coord = width_coord;
        let word_as_vec: Vec<char> = word.chars().collect();

        for letter_idx in 0..word_as_vec.len() {

            let letter = word_as_vec[letter_idx];
            let next_letter = word_as_vec.get(letter_idx + 1).copied();
            
            if let Err(e) = self.place_letter_at_index(height_coord, width_coord, letter, next_letter, WordDirection::Across) {
                self.clear_word(starting_width_coord, height_coord, width_coord, WordDirection::Across);
                return Err(e);
            }

            width_coord += 1;
        }

        width_coord -= 1;
        self.confirm_word(starting_width_coord, height_coord, width_coord, WordDirection::Across);
        self.placed_words.insert(word.clone());

        Ok(())
    }

    fn place_word_vertically_starting_at_given_coordinates(&mut self, mut height_coord: u32, width_coord: u32, word: &String) -> Result<(), WordPlacementError> {

        if word.len() as u32 > self.height - height_coord {
            return Err(WordPlacementError::DoesNotFitByHeight);
        }

        let starting_height_coord = height_coord;
        let word_as_vec: Vec<char> = word.chars().collect();

        for letter_idx in 0..word_as_vec.len() {

            let letter = word_as_vec[letter_idx];
            let next_letter = word_as_vec.get(letter_idx + 1).copied();     

            if let Err(e) = self.place_letter_at_index(height_coord, width_coord, letter, next_letter, WordDirection::Down) {
                self.clear_word(starting_height_coord, height_coord, width_coord, WordDirection::Down);
                return Err(e);
            }

            height_coord += 1
        }

        height_coord -= 1;
        self.confirm_word(starting_height_coord, height_coord, width_coord, WordDirection::Down);
        self.placed_words.insert(word.clone());

        Ok(())
    }

    fn place_letter_at_index(&mut self, height_coord: u32, width_coord: u32, letter: char, next_letter: Option<char>, direction: WordDirection) -> Result<(), WordPlacementError> {

        self.verify_current_cell(height_coord, width_coord, letter)?;
        self.verify_cell_to_the_left(height_coord, width_coord, &direction)?;
        self.verify_cell_above(height_coord, width_coord, &direction)?;
        self.verify_cell_to_the_right(height_coord, width_coord, next_letter, &direction)?;
        self.verify_cell_below(height_coord, width_coord, next_letter, &direction)?;

        if self.layout[height_coord as usize][width_coord as usize].cell_state == CellState::NotFilled {
            self.layout[height_coord as usize][width_coord as usize].cell_state = CellState::TentativelyFilled;
            self.layout[height_coord as usize][width_coord as usize].cell_value = letter;
        }

        Ok(())
    }

    fn reset_cell_at_index(&mut self, height_coord: u32, width_coord: u32) -> () {
        if self.layout[height_coord as usize][width_coord as usize].cell_state == CellState::TentativelyFilled {
            self.layout[height_coord as usize][width_coord as usize].reset();
        }

        else if self.layout[height_coord as usize][width_coord as usize].is_a_crossing == CrossingState::IsTentatively {
            self.layout[height_coord as usize][width_coord as usize].reset_crossing_state();
        }
        
        return;
    }

    fn confirm_cell_at_index(&mut self, height_coord: u32, width_coord: u32) -> () {
        self.layout[height_coord as usize][width_coord as usize].cell_state = CellState::Filled;

        if self.layout[height_coord as usize][width_coord as usize].is_a_crossing == CrossingState::IsTentatively {
            self.layout[height_coord as usize][width_coord as usize].is_a_crossing = CrossingState::Is;
        }

        return;
    }

    fn verify_current_cell(&mut self, height_coord: u32, width_coord: u32, letter: char) -> Result<(), WordPlacementError> {

        if self.layout[height_coord as usize][width_coord as usize].cell_state == CellState::NotFilled ||
            self.layout[height_coord as usize][width_coord as usize].cell_value == letter
        {
            if self.layout[height_coord as usize][width_coord as usize].cell_value == letter {
                self.layout[height_coord as usize][width_coord as usize].is_a_crossing = CrossingState::IsTentatively;
            }
            return Ok(());
        }

        Err(WordPlacementError::BlockerCurrent)
    }

    fn verify_cell_to_the_left(&self, height_coord: u32, width_coord: u32, direction: &WordDirection) -> Result<(), WordPlacementError> {

        if width_coord == 0 
        {
            return Ok(());
        }

        if self.layout[height_coord as usize][(width_coord - 1) as usize].cell_state != CellState::Filled || 
            (self.layout[height_coord as usize][(width_coord - 1) as usize].is_a_crossing == CrossingState::IsTentatively && *direction == WordDirection::Across) ||
            (self.layout[height_coord as usize][width_coord as usize].is_a_crossing == CrossingState::IsTentatively && *direction == WordDirection::Down)
        {
            return Ok(());
        }

        Err(WordPlacementError::BlockerLeft)
    }

    fn verify_cell_above(&self, height_coord: u32, width_coord: u32, direction: &WordDirection) -> Result<(), WordPlacementError> {

        if height_coord == 0 
        {
            return Ok(());
        }

        if self.layout[(height_coord - 1) as usize][width_coord as usize].cell_state != CellState::Filled ||
            (self.layout[(height_coord - 1) as usize][width_coord as usize].is_a_crossing == CrossingState::IsTentatively && *direction == WordDirection::Down) ||
            (self.layout[height_coord as usize][width_coord as usize].is_a_crossing == CrossingState::IsTentatively && *direction == WordDirection::Across)
        {
            return Ok(());
        }

        Err(WordPlacementError::BlockerTop)
    }

    fn verify_cell_to_the_right(&self, height_coord: u32, width_coord: u32, next_letter: Option<char>, direction: &WordDirection) -> Result<(), WordPlacementError> {

        if width_coord == self.width - 1 
        {
            return Ok(());
        }

        if *direction == WordDirection::Across && 
            self.layout[height_coord as usize][(width_coord + 1) as usize].cell_state == CellState::Filled &&
            next_letter.is_some() &&
            self.layout[height_coord as usize][(width_coord + 1) as usize].cell_value == next_letter.unwrap() &&
            width_coord + 1 == self.width - 1
        {
            return Ok(());
        }

        if *direction == WordDirection::Across && 
            self.layout[height_coord as usize][(width_coord + 1) as usize].cell_state == CellState::Filled &&
            width_coord + 1 != self.width - 1 &&
            self.layout[height_coord as usize][(width_coord + 2) as usize].cell_state == CellState::NotFilled &&
            next_letter.is_some() &&
            self.layout[height_coord as usize][(width_coord + 1) as usize].cell_value == next_letter.unwrap()
        {
            return Ok(());
        }

        if self.layout[height_coord as usize][(width_coord + 1) as usize].cell_state == CellState::NotFilled || 
            (self.layout[height_coord as usize][(width_coord + 1) as usize].cell_state == CellState::Filled && 
            self.layout[height_coord as usize][width_coord as usize].is_a_crossing == CrossingState::IsTentatively)
        {
            return Ok(());
        }

        Err(WordPlacementError::BlockerRight)
    }

    fn verify_cell_below(&self, height_coord: u32, width_coord: u32, next_letter: Option<char>, direction: &WordDirection) -> Result<(), WordPlacementError> {

        if height_coord == self.height - 1 
        {
            return Ok(());
        }

        if *direction == WordDirection::Down && 
            self.layout[(height_coord + 1) as usize][width_coord as usize].cell_state == CellState::Filled &&
            next_letter.is_some() &&
            self.layout[(height_coord + 1) as usize][width_coord as usize].cell_value == next_letter.unwrap() &&
            height_coord + 1 == self.height - 1
        {
            return Ok(());
        }

        if *direction == WordDirection::Down && 
            self.layout[(height_coord + 1) as usize][width_coord as usize].cell_state == CellState::Filled &&
            height_coord + 1 != self.height - 1 &&
            self.layout[(height_coord + 2) as usize][width_coord as usize].cell_state == CellState::NotFilled &&
            next_letter.is_some() &&
            self.layout[(height_coord + 1) as usize][width_coord as usize].cell_value == next_letter.unwrap()
        {
            return Ok(());
        }

        if self.layout[(height_coord + 1) as usize][width_coord as usize].cell_state == CellState::NotFilled ||
            (self.layout[(height_coord + 1) as usize][width_coord as usize].cell_state == CellState::Filled && 
            self.layout[height_coord as usize][width_coord as usize].is_a_crossing == CrossingState::IsTentatively)
        {
            return Ok(());
        }

        Err(WordPlacementError::BlockerBottom)
    }
}
