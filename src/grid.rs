use num::Integer;
use std::collections::HashSet;

#[cfg(test)]
mod unit_tests;

#[derive(PartialEq, Debug)]
enum CellState {
    Filled,
    NotFilled,
    TentativelyFilled
}

#[derive(PartialEq)]
enum CrossingState {
    Is,
    IsTentatively,
    IsNot
}

struct GridCell {
    cell_value: char,
    cell_state: CellState,
    is_a_crossing: CrossingState
}

impl GridCell {
    fn new() -> Self {
        Self { cell_value: '_', cell_state: CellState::NotFilled, is_a_crossing: CrossingState::IsNot }
    }

    fn reset(&mut self) {
        self.cell_value = '_';
        self.cell_state = CellState::NotFilled;
        self.reset_crossing_state();
    }

    fn reset_crossing_state(&mut self) {
        self.is_a_crossing = CrossingState::IsNot;
    }
}

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

#[derive(PartialEq)]
enum WordDirection {
    Across,
    Down
}

pub struct Grid {
    width: u32,
    height: u32,
    layout: Vec<Vec<GridCell>>,
    placed_words: HashSet<String>
}

impl Grid {
    pub fn new(width: u32, height: u32) -> Self {
        let mut layout : Vec<Vec<GridCell>> = Vec::new();

        for row_idx in 0..height {
            layout.push(Vec::new());

            for _ in 0..width {
                let cell = GridCell::new();
                layout[row_idx as usize].push(cell);
            }
        }

        let placed_words = HashSet::new();

        Grid { width, height, layout, placed_words }
    }

    pub fn print(&self) {
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

    pub fn place_word(&mut self, word: &String) -> Result<(), WordPlacementError> {

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

                while width_coord >= starting_width_coord {
                    self.reset_cell_at_index(height_coord, width_coord);

                    if width_coord == 0 {
                        break;
                    }

                    if width_coord > 0 {
                        width_coord -= 1;
                    }
                }

                return Err(e);
            }

            width_coord += 1;
        }

        width_coord -= 1;

        while width_coord >= starting_width_coord {

            self.confirm_cell_at_index(height_coord, width_coord);
            
            if width_coord == 0 {
                break;
            } else {
                width_coord -= 1;
            }
        }

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

                while height_coord >= starting_height_coord {
                    self.reset_cell_at_index(height_coord, width_coord);

                    if height_coord == 0 {
                        break;
                    }

                    if height_coord > 0 {
                        height_coord -= 1;
                    }
                }

                return Err(e);
            }

            height_coord += 1
        }

        height_coord -= 1;

        while height_coord >= starting_height_coord {
            self.confirm_cell_at_index(height_coord, width_coord);

            if height_coord == 0 {
                break;
            } else {
                height_coord -= 1;
            }
        }

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
