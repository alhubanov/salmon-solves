use num::Integer;
use std::collections::HashSet;

#[derive(PartialEq)]
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

#[cfg(test)]
mod tests {
    use crate::grid::*;

    #[test]
    fn cell_reset() {

        let mut grid = Grid::new(1, 1);

        // case Filled
        grid.layout[0 as usize][0 as usize].cell_value = 'a';
        grid.layout[0 as usize][0 as usize].cell_state = CellState::Filled;

        grid.reset_cell_at_index(0, 0);

        assert!(grid.layout[0 as usize][0 as usize].cell_value == 'a');
        assert!(grid.layout[0 as usize][0 as usize].cell_state == CellState::Filled);

        // case TentativelyFilled
        grid.layout[0 as usize][0 as usize].cell_value = '1';
        grid.layout[0 as usize][0 as usize].cell_state = CellState::TentativelyFilled;

        grid.reset_cell_at_index(0, 0);

        assert!(grid.layout[0 as usize][0 as usize].cell_value == '_');
        assert!(grid.layout[0 as usize][0 as usize].cell_state == CellState::NotFilled);

        // case Empty
        grid.layout[0 as usize][0 as usize].cell_value = '_';
        grid.layout[0 as usize][0 as usize].cell_state = CellState::NotFilled;

        grid.reset_cell_at_index(0, 0);

        assert!(grid.layout[0 as usize][0 as usize].cell_value == '_');
        assert!(grid.layout[0 as usize][0 as usize].cell_state == CellState::NotFilled);
    }

    #[test]
    fn cell_confirmation() {
        let mut grid = Grid::new(1, 1);

        // case Filled
        grid.layout[0 as usize][0 as usize].cell_value = 'a';
        grid.layout[0 as usize][0 as usize].cell_state = CellState::Filled;

        grid.confirm_cell_at_index(0, 0);

        assert!(grid.layout[0 as usize][0 as usize].cell_value == 'a');
        assert!(grid.layout[0 as usize][0 as usize].cell_state == CellState::Filled);

        // case TentativelyFilled
        grid.layout[0 as usize][0 as usize].cell_value = '1';
        grid.layout[0 as usize][0 as usize].cell_state = CellState::TentativelyFilled;

        grid.confirm_cell_at_index(0, 0);

        assert!(grid.layout[0 as usize][0 as usize].cell_value == '1');
        assert!(grid.layout[0 as usize][0 as usize].cell_state == CellState::Filled);

        // case Empty
        grid.layout[0 as usize][0 as usize].cell_value = 'q';
        grid.layout[0 as usize][0 as usize].cell_state = CellState::NotFilled;

        grid.confirm_cell_at_index(0, 0);

        assert!(grid.layout[0 as usize][0 as usize].cell_value == 'q');
        assert!(grid.layout[0 as usize][0 as usize].cell_state == CellState::Filled);
    }

    #[test]
    fn placing_words_horizontally() {
        let mut grid = Grid::new(4, 4);
        let result = grid.place_word_horizontally_starting_at_given_coordinates(0, 0, &"word".to_owned());

        assert!(result.is_ok());
        assert!(grid.layout[0 as usize][0 as usize].cell_value == 'w');
        assert!(grid.layout[0 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][1 as usize].cell_value == 'o');
        assert!(grid.layout[0 as usize][1 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][2 as usize].cell_value == 'r');
        assert!(grid.layout[0 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][3 as usize].cell_value == 'd');
        assert!(grid.layout[0 as usize][3 as usize].cell_state == CellState::Filled);

        let result = grid.place_word_horizontally_starting_at_given_coordinates(0, 1, &"the".to_owned());

        assert!(result.is_err());
        assert!(grid.layout[0 as usize][0 as usize].cell_value == 'w');
        assert!(grid.layout[0 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][1 as usize].cell_value == 'o');
        assert!(grid.layout[0 as usize][1 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][2 as usize].cell_value == 'r');
        assert!(grid.layout[0 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][3 as usize].cell_value == 'd');
        assert!(grid.layout[0 as usize][3 as usize].cell_state == CellState::Filled);

        let result = grid.place_word_horizontally_starting_at_given_coordinates(0, 0, &"moat".to_owned());

        assert!(result.is_err());
        assert!(grid.layout[0 as usize][0 as usize].cell_value == 'w');
        assert!(grid.layout[0 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][1 as usize].cell_value == 'o');
        assert!(grid.layout[0 as usize][1 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][2 as usize].cell_value == 'r');
        assert!(grid.layout[0 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][3 as usize].cell_value == 'd');
        assert!(grid.layout[0 as usize][3 as usize].cell_state == CellState::Filled);
    }

    #[test]
    fn placing_words_horizontally_with_crossing_at_start() {
        let mut grid = Grid::new(5, 5);
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
        assert!(grid.layout[0 as usize][0 as usize].cell_value == 'm');
        assert!(grid.layout[0 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][1 as usize].cell_value == 'o');
        assert!(grid.layout[0 as usize][1 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][2 as usize].cell_value == 'a');
        assert!(grid.layout[0 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][3 as usize].cell_value == 't');
        assert!(grid.layout[0 as usize][3 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][0 as usize].cell_value == 'm');
        assert!(grid.layout[0 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[1 as usize][0 as usize].cell_value == 'a');
        assert!(grid.layout[1 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][0 as usize].cell_value == 'p');
        assert!(grid.layout[2 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[3 as usize][0 as usize].cell_value == 'l');
        assert!(grid.layout[3 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][0 as usize].cell_value == 'e');
        assert!(grid.layout[4 as usize][0 as usize].cell_state == CellState::Filled);

        let result = grid.place_word_horizontally_starting_at_given_coordinates(2, 0, &"star".to_owned());
        assert!(result.is_err());

        let result = grid.place_word_horizontally_starting_at_given_coordinates(2, 0, &"post".to_owned());
        assert!(result.is_ok());
        assert!(grid.layout[2 as usize][0 as usize].cell_value == 'p');
        assert!(grid.layout[2 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][1 as usize].cell_value == 'o');
        assert!(grid.layout[2 as usize][1 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][2 as usize].cell_value == 's');
        assert!(grid.layout[2 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][3 as usize].cell_value == 't');
        assert!(grid.layout[2 as usize][3 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][0 as usize].cell_value == 'm');
        assert!(grid.layout[0 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[1 as usize][0 as usize].cell_value == 'a');
        assert!(grid.layout[1 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][0 as usize].cell_value == 'p');
        assert!(grid.layout[2 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[3 as usize][0 as usize].cell_value == 'l');
        assert!(grid.layout[3 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][0 as usize].cell_value == 'e');
        assert!(grid.layout[4 as usize][0 as usize].cell_state == CellState::Filled);

        let result = grid.place_word_horizontally_starting_at_given_coordinates(4, 0, &"star".to_owned());
        assert!(result.is_err());

        let result = grid.place_word_horizontally_starting_at_given_coordinates(4, 0, &"east".to_owned());
        assert!(result.is_ok());
        assert!(grid.layout[4 as usize][0 as usize].cell_value == 'e');
        assert!(grid.layout[4 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][1 as usize].cell_value == 'a');
        assert!(grid.layout[4 as usize][1 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][2 as usize].cell_value == 's');
        assert!(grid.layout[4 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][3 as usize].cell_value == 't');
        assert!(grid.layout[4 as usize][3 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][0 as usize].cell_value == 'm');
        assert!(grid.layout[0 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[1 as usize][0 as usize].cell_value == 'a');
        assert!(grid.layout[1 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][0 as usize].cell_value == 'p');
        assert!(grid.layout[2 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[3 as usize][0 as usize].cell_value == 'l');
        assert!(grid.layout[3 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][0 as usize].cell_value == 'e');
        assert!(grid.layout[4 as usize][0 as usize].cell_state == CellState::Filled);

        assert!(grid.placed_words.len() == 3);
    }

    #[test]
    fn placing_words_horizontally_with_crossing_in_the_middle() {
        let mut grid = Grid::new(5, 5);
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
        assert!(grid.layout[0 as usize][0 as usize].cell_value == 'm');
        assert!(grid.layout[0 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][1 as usize].cell_value == 'o');
        assert!(grid.layout[0 as usize][1 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][2 as usize].cell_value == 'm');
        assert!(grid.layout[0 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][3 as usize].cell_value == 'm');
        assert!(grid.layout[0 as usize][3 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][4 as usize].cell_value == 'a');
        assert!(grid.layout[0 as usize][4 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][2 as usize].cell_value == 'm');
        assert!(grid.layout[0 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[1 as usize][2 as usize].cell_value == 'a');
        assert!(grid.layout[1 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][2 as usize].cell_value == 'p');
        assert!(grid.layout[2 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[3 as usize][2 as usize].cell_value == 'l');
        assert!(grid.layout[3 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][2 as usize].cell_value == 'e');
        assert!(grid.layout[4 as usize][2 as usize].cell_state == CellState::Filled);

        let result = grid.place_word_horizontally_starting_at_given_coordinates(2, 0, &"star".to_owned());
        assert!(result.is_err());

        let result = grid.place_word_horizontally_starting_at_given_coordinates(2, 0, &"cops".to_owned());
        assert!(result.is_ok());
        assert!(grid.layout[2 as usize][0 as usize].cell_value == 'c');
        assert!(grid.layout[2 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][1 as usize].cell_value == 'o');
        assert!(grid.layout[2 as usize][1 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][2 as usize].cell_value == 'p');
        assert!(grid.layout[2 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][3 as usize].cell_value == 's');
        assert!(grid.layout[2 as usize][3 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][2 as usize].cell_value == 'm');
        assert!(grid.layout[0 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[1 as usize][2 as usize].cell_value == 'a');
        assert!(grid.layout[1 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][2 as usize].cell_value == 'p');
        assert!(grid.layout[2 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[3 as usize][2 as usize].cell_value == 'l');
        assert!(grid.layout[3 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][2 as usize].cell_value == 'e');
        assert!(grid.layout[4 as usize][2 as usize].cell_state == CellState::Filled);

        let result = grid.place_word_horizontally_starting_at_given_coordinates(4, 0, &"star".to_owned());
        assert!(result.is_err());

        let result = grid.place_word_horizontally_starting_at_given_coordinates(4, 0, &"fred".to_owned());
        assert!(result.is_ok());
        assert!(grid.layout[4 as usize][0 as usize].cell_value == 'f');
        assert!(grid.layout[4 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][1 as usize].cell_value == 'r');
        assert!(grid.layout[4 as usize][1 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][2 as usize].cell_value == 'e');
        assert!(grid.layout[4 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][3 as usize].cell_value == 'd');
        assert!(grid.layout[4 as usize][3 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][2 as usize].cell_value == 'm');
        assert!(grid.layout[0 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[1 as usize][2 as usize].cell_value == 'a');
        assert!(grid.layout[1 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][2 as usize].cell_value == 'p');
        assert!(grid.layout[2 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[3 as usize][2 as usize].cell_value == 'l');
        assert!(grid.layout[3 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][2 as usize].cell_value == 'e');
        assert!(grid.layout[4 as usize][2 as usize].cell_state == CellState::Filled);

        assert!(grid.placed_words.len() == 3);
    }

    #[test]
    fn placing_words_horizontally_with_crossing_in_the_end() {
        let mut grid = Grid::new(5, 5);
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
        assert!(grid.layout[0 as usize][0 as usize].cell_value == 'b');
        assert!(grid.layout[0 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][1 as usize].cell_value == 'i');
        assert!(grid.layout[0 as usize][1 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][2 as usize].cell_value == 't');
        assert!(grid.layout[0 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][3 as usize].cell_value == 'u');
        assert!(grid.layout[0 as usize][3 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][4 as usize].cell_value == 'm');
        assert!(grid.layout[0 as usize][4 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][4 as usize].cell_value == 'm');
        assert!(grid.layout[0 as usize][4 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[1 as usize][4 as usize].cell_value == 'a');
        assert!(grid.layout[1 as usize][4 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][4 as usize].cell_value == 'p');
        assert!(grid.layout[2 as usize][4 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[3 as usize][4 as usize].cell_value == 'l');
        assert!(grid.layout[3 as usize][4 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][4 as usize].cell_value == 'e');
        assert!(grid.layout[4 as usize][4 as usize].cell_state == CellState::Filled);

        let result = grid.place_word_horizontally_starting_at_given_coordinates(2, 0, &"start".to_owned());
        assert!(result.is_err());

        let result = grid.place_word_horizontally_starting_at_given_coordinates(2, 0, &"aesop".to_owned());
        assert!(result.is_ok());
        assert!(grid.layout[2 as usize][0 as usize].cell_value == 'a');
        assert!(grid.layout[2 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][1 as usize].cell_value == 'e');
        assert!(grid.layout[2 as usize][1 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][2 as usize].cell_value == 's');
        assert!(grid.layout[2 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][3 as usize].cell_value == 'o');
        assert!(grid.layout[2 as usize][3 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][4 as usize].cell_value == 'p');
        assert!(grid.layout[2 as usize][4 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][4 as usize].cell_value == 'm');
        assert!(grid.layout[0 as usize][4 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[1 as usize][4 as usize].cell_value == 'a');
        assert!(grid.layout[1 as usize][4 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][4 as usize].cell_value == 'p');
        assert!(grid.layout[2 as usize][4 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[3 as usize][4 as usize].cell_value == 'l');
        assert!(grid.layout[3 as usize][4 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][4 as usize].cell_value == 'e');
        assert!(grid.layout[4 as usize][4 as usize].cell_state == CellState::Filled);

        let result = grid.place_word_horizontally_starting_at_given_coordinates(4, 0, &"start".to_owned());
        assert!(result.is_err());

        let result = grid.place_word_horizontally_starting_at_given_coordinates(4, 0, &"andre".to_owned());
        assert!(result.is_ok());
        assert!(grid.layout[4 as usize][0 as usize].cell_value == 'a');
        assert!(grid.layout[4 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][1 as usize].cell_value == 'n');
        assert!(grid.layout[4 as usize][1 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][2 as usize].cell_value == 'd');
        assert!(grid.layout[4 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][3 as usize].cell_value == 'r');
        assert!(grid.layout[4 as usize][3 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][4 as usize].cell_value == 'e');
        assert!(grid.layout[4 as usize][4 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][4 as usize].cell_value == 'm');
        assert!(grid.layout[0 as usize][4 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[1 as usize][4 as usize].cell_value == 'a');
        assert!(grid.layout[1 as usize][4 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][4 as usize].cell_value == 'p');
        assert!(grid.layout[2 as usize][4 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[3 as usize][4 as usize].cell_value == 'l');
        assert!(grid.layout[3 as usize][4 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][4 as usize].cell_value == 'e');
        assert!(grid.layout[4 as usize][4 as usize].cell_state == CellState::Filled);

        assert!(grid.placed_words.len() == 3);
    }

    #[test]
    fn placing_words_vertically() {
        let mut grid = Grid::new(4, 4);
        let result = grid.place_word_vertically_starting_at_given_coordinates(0, 0, &"word".to_owned());

        assert!(result.is_ok());
        assert!(grid.layout[0 as usize][0 as usize].cell_value == 'w');
        assert!(grid.layout[0 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[1 as usize][0 as usize].cell_value == 'o');
        assert!(grid.layout[1 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][0 as usize].cell_value == 'r');
        assert!(grid.layout[2 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[3 as usize][0 as usize].cell_value == 'd');
        assert!(grid.layout[3 as usize][0 as usize].cell_state == CellState::Filled);

        let result = grid.place_word_vertically_starting_at_given_coordinates(1, 0, &"the".to_owned());

        assert!(result.is_err());
        assert!(grid.layout[0 as usize][0 as usize].cell_value == 'w');
        assert!(grid.layout[0 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[1 as usize][0 as usize].cell_value == 'o');
        assert!(grid.layout[1 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][0 as usize].cell_value == 'r');
        assert!(grid.layout[2 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[3 as usize][0 as usize].cell_value == 'd');
        assert!(grid.layout[3 as usize][0 as usize].cell_state == CellState::Filled);

        let result = grid.place_word_vertically_starting_at_given_coordinates(0, 0, &"moat".to_owned());

        assert!(result.is_err());
        assert!(grid.layout[0 as usize][0 as usize].cell_value == 'w');
        assert!(grid.layout[0 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[1 as usize][0 as usize].cell_value == 'o');
        assert!(grid.layout[1 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][0 as usize].cell_value == 'r');
        assert!(grid.layout[2 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[3 as usize][0 as usize].cell_value == 'd');
        assert!(grid.layout[3 as usize][0 as usize].cell_state == CellState::Filled);
    }

    #[test]
    fn placing_words_vertically_with_crossing_at_start() {
        let mut grid = Grid::new(5, 5);
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
        assert!(grid.layout[0 as usize][0 as usize].cell_value == 'm');
        assert!(grid.layout[0 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[1 as usize][0 as usize].cell_value == 'o');
        assert!(grid.layout[1 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][0 as usize].cell_value == 'a');
        assert!(grid.layout[2 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[3 as usize][0 as usize].cell_value == 't');
        assert!(grid.layout[3 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][0 as usize].cell_value == 'm');
        assert!(grid.layout[0 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][1 as usize].cell_value == 'a');
        assert!(grid.layout[0 as usize][1 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][2 as usize].cell_value == 'p');
        assert!(grid.layout[0 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][3 as usize].cell_value == 'l');
        assert!(grid.layout[0 as usize][3 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][4 as usize].cell_value == 'e');
        assert!(grid.layout[0 as usize][4 as usize].cell_state == CellState::Filled);

        let result = grid.place_word_vertically_starting_at_given_coordinates(0, 2, &"star".to_owned());
        assert!(result.is_err());

        let result = grid.place_word_vertically_starting_at_given_coordinates(0, 2, &"post".to_owned());
        assert!(result.is_ok());
        assert!(grid.layout[0 as usize][2 as usize].cell_value == 'p');
        assert!(grid.layout[0 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[1 as usize][2 as usize].cell_value == 'o');
        assert!(grid.layout[1 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][2 as usize].cell_value == 's');
        assert!(grid.layout[2 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[3 as usize][2 as usize].cell_value == 't');
        assert!(grid.layout[3 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][0 as usize].cell_value == 'm');
        assert!(grid.layout[0 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][1 as usize].cell_value == 'a');
        assert!(grid.layout[0 as usize][1 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][2 as usize].cell_value == 'p');
        assert!(grid.layout[0 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][3 as usize].cell_value == 'l');
        assert!(grid.layout[0 as usize][3 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][4 as usize].cell_value == 'e');
        assert!(grid.layout[0 as usize][4 as usize].cell_state == CellState::Filled);

        let result = grid.place_word_vertically_starting_at_given_coordinates(0, 4, &"star".to_owned());
        assert!(result.is_err());

        let result = grid.place_word_vertically_starting_at_given_coordinates(0, 4, &"east".to_owned());
        assert!(result.is_ok());
        assert!(grid.layout[0 as usize][4 as usize].cell_value == 'e');
        assert!(grid.layout[0 as usize][4 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[1 as usize][4 as usize].cell_value == 'a');
        assert!(grid.layout[1 as usize][4 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][4 as usize].cell_value == 's');
        assert!(grid.layout[2 as usize][4 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[3 as usize][4 as usize].cell_value == 't');
        assert!(grid.layout[3 as usize][4 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][0 as usize].cell_value == 'm');
        assert!(grid.layout[0 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][1 as usize].cell_value == 'a');
        assert!(grid.layout[0 as usize][1 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][2 as usize].cell_value == 'p');
        assert!(grid.layout[0 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][3 as usize].cell_value == 'l');
        assert!(grid.layout[0 as usize][3 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[0 as usize][4 as usize].cell_value == 'e');
        assert!(grid.layout[0 as usize][4 as usize].cell_state == CellState::Filled);

        assert!(grid.placed_words.len() == 3);
    }

    #[test]
    fn placing_words_vertically_with_crossing_in_the_middle() {
        let mut grid = Grid::new(5, 5);
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
        assert!(grid.layout[0 as usize][0 as usize].cell_value == 'm');
        assert!(grid.layout[0 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[1 as usize][0 as usize].cell_value == 'o');
        assert!(grid.layout[1 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][0 as usize].cell_value == 'm');
        assert!(grid.layout[2 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[3 as usize][0 as usize].cell_value == 'm');
        assert!(grid.layout[3 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][0 as usize].cell_value == 'a');
        assert!(grid.layout[4 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][0 as usize].cell_value == 'm');
        assert!(grid.layout[2 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][1 as usize].cell_value == 'a');
        assert!(grid.layout[2 as usize][1 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][2 as usize].cell_value == 'p');
        assert!(grid.layout[2 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][3 as usize].cell_value == 'l');
        assert!(grid.layout[2 as usize][3 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][4 as usize].cell_value == 'e');
        assert!(grid.layout[2 as usize][4 as usize].cell_state == CellState::Filled);

        let result = grid.place_word_vertically_starting_at_given_coordinates(0, 2, &"star".to_owned());
        assert!(result.is_err());

        let result = grid.place_word_vertically_starting_at_given_coordinates(0, 2, &"cops".to_owned());
        assert!(result.is_ok());
        assert!(grid.layout[0 as usize][2 as usize].cell_value == 'c');
        assert!(grid.layout[0 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[1 as usize][2 as usize].cell_value == 'o');
        assert!(grid.layout[1 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][2 as usize].cell_value == 'p');
        assert!(grid.layout[2 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[3 as usize][2 as usize].cell_value == 's');
        assert!(grid.layout[3 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][0 as usize].cell_value == 'm');
        assert!(grid.layout[2 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][1 as usize].cell_value == 'a');
        assert!(grid.layout[2 as usize][1 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][2 as usize].cell_value == 'p');
        assert!(grid.layout[2 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][3 as usize].cell_value == 'l');
        assert!(grid.layout[2 as usize][3 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][4 as usize].cell_value == 'e');
        assert!(grid.layout[2 as usize][4 as usize].cell_state == CellState::Filled);

        let result = grid.place_word_vertically_starting_at_given_coordinates(0, 4, &"star".to_owned());
        assert!(result.is_err());

        let result = grid.place_word_vertically_starting_at_given_coordinates(0, 4, &"fred".to_owned());
        assert!(result.is_ok());
        assert!(grid.layout[0 as usize][4 as usize].cell_value == 'f');
        assert!(grid.layout[0 as usize][4 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[1 as usize][4 as usize].cell_value == 'r');
        assert!(grid.layout[1 as usize][4 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][4 as usize].cell_value == 'e');
        assert!(grid.layout[2 as usize][4 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[3 as usize][4 as usize].cell_value == 'd');
        assert!(grid.layout[3 as usize][4 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][0 as usize].cell_value == 'm');
        assert!(grid.layout[2 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][1 as usize].cell_value == 'a');
        assert!(grid.layout[2 as usize][1 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][2 as usize].cell_value == 'p');
        assert!(grid.layout[2 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][3 as usize].cell_value == 'l');
        assert!(grid.layout[2 as usize][3 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][4 as usize].cell_value == 'e');
        assert!(grid.layout[2 as usize][4 as usize].cell_state == CellState::Filled);

        assert!(grid.placed_words.len() == 3);
    }

    #[test]
    fn placing_words_vertically_with_crossing_in_the_end() {
        let mut grid = Grid::new(5, 5);
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
        assert!(grid.layout[0 as usize][0 as usize].cell_value == 'b');
        assert!(grid.layout[0 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[1 as usize][0 as usize].cell_value == 'i');
        assert!(grid.layout[1 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][0 as usize].cell_value == 't');
        assert!(grid.layout[2 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[3 as usize][0 as usize].cell_value == 'u');
        assert!(grid.layout[3 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][0 as usize].cell_value == 'm');
        assert!(grid.layout[4 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][0 as usize].cell_value == 'm');
        assert!(grid.layout[4 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][1 as usize].cell_value == 'a');
        assert!(grid.layout[4 as usize][1 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][2 as usize].cell_value == 'p');
        assert!(grid.layout[4 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][3 as usize].cell_value == 'l');
        assert!(grid.layout[4 as usize][3 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][4 as usize].cell_value == 'e');
        assert!(grid.layout[4 as usize][4 as usize].cell_state == CellState::Filled);

        let result = grid.place_word_vertically_starting_at_given_coordinates(0, 2, &"start".to_owned());
        assert!(result.is_err());

        let result = grid.place_word_vertically_starting_at_given_coordinates(0, 2, &"aesop".to_owned());
        assert!(result.is_ok());
        assert!(grid.layout[0 as usize][2 as usize].cell_value == 'a');
        assert!(grid.layout[0 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[1 as usize][2 as usize].cell_value == 'e');
        assert!(grid.layout[1 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][2 as usize].cell_value == 's');
        assert!(grid.layout[2 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[3 as usize][2 as usize].cell_value == 'o');
        assert!(grid.layout[3 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][2 as usize].cell_value == 'p');
        assert!(grid.layout[4 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][0 as usize].cell_value == 'm');
        assert!(grid.layout[4 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][1 as usize].cell_value == 'a');
        assert!(grid.layout[4 as usize][1 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][2 as usize].cell_value == 'p');
        assert!(grid.layout[4 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][3 as usize].cell_value == 'l');
        assert!(grid.layout[4 as usize][3 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][4 as usize].cell_value == 'e');
        assert!(grid.layout[4 as usize][4 as usize].cell_state == CellState::Filled);

        let result = grid.place_word_vertically_starting_at_given_coordinates(0, 4, &"start".to_owned());
        assert!(result.is_err());

        let result = grid.place_word_vertically_starting_at_given_coordinates(0, 4, &"andre".to_owned());
        assert!(result.is_ok());
        assert!(grid.layout[0 as usize][4 as usize].cell_value == 'a');
        assert!(grid.layout[0 as usize][4 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[1 as usize][4 as usize].cell_value == 'n');
        assert!(grid.layout[1 as usize][4 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][4 as usize].cell_value == 'd');
        assert!(grid.layout[2 as usize][4 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[3 as usize][4 as usize].cell_value == 'r');
        assert!(grid.layout[3 as usize][4 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][4 as usize].cell_value == 'e');
        assert!(grid.layout[4 as usize][4 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][0 as usize].cell_value == 'm');
        assert!(grid.layout[4 as usize][0 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][1 as usize].cell_value == 'a');
        assert!(grid.layout[4 as usize][1 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][2 as usize].cell_value == 'p');
        assert!(grid.layout[4 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][3 as usize].cell_value == 'l');
        assert!(grid.layout[4 as usize][3 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[4 as usize][4 as usize].cell_value == 'e');
        assert!(grid.layout[4 as usize][4 as usize].cell_state == CellState::Filled);

        assert!(grid.placed_words.len() == 3);
    }

    #[test]
    fn placing_words_out_of_bounds() {
        let mut grid = Grid::new(5, 5);

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
        let mut grid = Grid::new(7, 7);

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
        let mut grid = Grid::new(7, 7);

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
        let mut grid = Grid::new(5, 5);

        let result = grid.place_word_horizontally_starting_at_given_coordinates(2, 0, &"maple".to_owned());
        assert!(result.is_ok());

        let result = grid.place_word_vertically_starting_at_given_coordinates(0, 2, &"baste".to_owned());
        assert!(result.is_err());

        assert!(grid.placed_words.len() == 1);
    }

    #[test]
    fn impossible_crossing_2() {
        let mut grid = Grid::new(5, 5);

        let result = grid.place_word_vertically_starting_at_given_coordinates(0, 2, &"baste".to_owned());
        assert!(result.is_ok());

        let result = grid.place_word_horizontally_starting_at_given_coordinates(2, 0, &"maple".to_owned());
        assert!(result.is_err());

        assert!(grid.placed_words.len() == 1);
    }

    #[test]
    fn placing_first_word_1() {
        let mut grid = Grid::new(5, 5);

        let result = grid.place_first_word(&"bass".to_owned());
        assert!(result.is_ok());

        assert!(grid.layout[2 as usize][1 as usize].cell_value == 'b');
        assert!(grid.layout[2 as usize][1 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][2 as usize].cell_value == 'a');
        assert!(grid.layout[2 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][3 as usize].cell_value == 's');
        assert!(grid.layout[2 as usize][3 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][4 as usize].cell_value == 's');
        assert!(grid.layout[2 as usize][4 as usize].cell_state == CellState::Filled);

        assert!(grid.placed_words.len() == 1);
    }

    #[test]
    fn placing_first_word_2() {
        let mut grid = Grid::new(6, 6);

        let result = grid.place_first_word(&"bass".to_owned());
        assert!(result.is_ok());

        assert!(grid.layout[2 as usize][1 as usize].cell_value == 'b');
        assert!(grid.layout[2 as usize][1 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][2 as usize].cell_value == 'a');
        assert!(grid.layout[2 as usize][2 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][3 as usize].cell_value == 's');
        assert!(grid.layout[2 as usize][3 as usize].cell_state == CellState::Filled);
        assert!(grid.layout[2 as usize][4 as usize].cell_value == 's');
        assert!(grid.layout[2 as usize][4 as usize].cell_state == CellState::Filled);

        assert!(grid.placed_words.len() == 1);
    }
}