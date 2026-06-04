use std::cmp::max;
use std::{collections::BTreeSet};
use rand::prelude::*;

#[cfg(test)]
mod unit_tests;
#[cfg(test)]
mod test_helpers;

mod clue;
mod letter;
mod gridcell;
mod slot;

use slot::Slot;
use slot::SlotDirection;
use gridcell::GridCell;
use crate::grid_scandi::gridcell::PossibleCellState;
use crate::grid::Grid;

mod constants {
    pub const TARGET_CLUE_DENSITY : f32                        = 0.25;
    pub const THRESHOLD_PROBABILITY_FOR_SECOND_CLUE_STATE: f32 = 0.6;
    pub const _MAXIMUM_WORD_LENGTH : i32                       = 10; // unused currently
    pub const MEDIAN_WORD_LENGTH : i32                         = 6; 
    pub const PER_LETTER_LENGTH_PRESSURE : f32                 = 0.08;
    pub const _DEFICIT_RATE_DEFAULT : f32                      = 0.35; // unused currently
}

pub enum LayoutError {
    // NoPossibleDomain,
    // LowClueDensity,
    // HighClueDensity,
    // IsolatedLetter,
    CannotEnsureAClearWordSlot
}

pub struct ScandiGrid {
    width: u32,
    height: u32,
    layout: Vec<Vec<GridCell>>,
    clues_placed: u32,
    num_cells_accessed: u32,
    word_slots: Vec<Slot<'static>>
}

impl Grid for ScandiGrid {
    fn initialize(width: u32, height: u32) -> Self {
        let mut layout : Vec<Vec<GridCell>> = Vec::new();

        for row_idx in 0..height {
            layout.push(Vec::new());

            for col_idx in 0..width {
                let cell = GridCell::build(row_idx, col_idx, width, height);

                layout[row_idx as usize].push(cell);
            }
        }
        
        let num_cells_accessed = 0;
        let clues_placed = 0;

        let word_slots = Vec::new();

        Self { width, height, layout, clues_placed, num_cells_accessed, word_slots }
    }

    fn construct(&mut self) -> () {
        for vertical_idx in 0..self.height {
            for horizontal_idx in 0..self.width {

                let assigned_cell_states = self.set_cell_state_from_remaining_possibilities(vertical_idx, horizontal_idx);
                if let Err(_) = self.restrict_neighborhood(vertical_idx, horizontal_idx, &assigned_cell_states) {
                    // backtrack
                }
            }
        }
    }

    fn print(&self) -> () {
        for vertical_idx in 0..self.height {
            for horizontal_idx in 0..self.width {
                self.layout[vertical_idx as usize][horizontal_idx as usize].print();
            }

            println!()
        }
    }
}

impl ScandiGrid {

    fn set_cell_state_from_remaining_possibilities(&mut self, vertical_idx: u32, horizontal_idx: u32) -> BTreeSet<PossibleCellState> {

        self.apply_first_row_column_restrictions(vertical_idx, horizontal_idx);
        let current_word_len : i32 = self.word_len_up_to_curr_cell(vertical_idx, horizontal_idx);

        let curr_cell = &mut self.layout[vertical_idx as usize][horizontal_idx as usize];
        let mut rng = rand::rng();
        let assigned_states : BTreeSet<PossibleCellState> = BTreeSet::new();

        if curr_cell.can_still_be_clue() && curr_cell.can_still_be_letter() 
        {
            // the ratio betwee placed clues and processed cells, not considering the first row and first column
            let current_density = if self.num_cells_accessed > 0 { self.clues_placed as f32 / self.num_cells_accessed as f32 } else { 0.0 };

            // force towards a clue if falling behind target clue density 
            let deficit_rate = if constants::TARGET_CLUE_DENSITY > current_density { (constants::TARGET_CLUE_DENSITY - current_density) * 2.0 } else { 0.0 };

            // force towards a clue or a letter depending on current length
            let length_pressure : f32 = (current_word_len - constants::MEDIAN_WORD_LENGTH) as f32 * constants::PER_LETTER_LENGTH_PRESSURE;

            let mut clue_probability = (constants::TARGET_CLUE_DENSITY + deficit_rate + length_pressure).clamp(0.0, 1.0);
            clue_probability = clue_probability.clamp(0.00, 1.0);

            let random_num :f32 = rng.random();
            if random_num as f32 <= clue_probability 
            {
                self.clues_placed += 1;
                self.num_cells_accessed += 1;
                return Self::assign_clue_states(curr_cell, rng, assigned_states, false);
            } 
            else 
            {
                self.num_cells_accessed += 1;
                return Self::assign_letter(curr_cell, assigned_states); 
            }
        } 
        else if (horizontal_idx == 0 || vertical_idx == 0) && curr_cell.can_still_be_clue() 
        {
            // self.clues_placed += 1;
            return Self::assign_clue_states(curr_cell, rng, assigned_states, true);
        }
        else if curr_cell.can_still_be_clue() 
        {
            self.num_cells_accessed += 1;
            self.clues_placed += 1;
            return Self::assign_clue_states(curr_cell, rng, assigned_states, false);
        } 
        else if curr_cell.can_still_be_letter() 
        {
            self.num_cells_accessed += 1;
            return Self::assign_letter(curr_cell, assigned_states); 
        } 
        else 
        {
            panic!("Reached point of assigning state with no available possibilities.")
        }
    }

    fn word_len_up_to_curr_cell(&self, vertical_idx: u32, horizontal_idx: u32) -> i32 {
        let mut slot_count = -1;
        let mut curr_vertical_idx = vertical_idx;
        let mut curr_horizontal_idx = horizontal_idx;

        let vertical_word_len = loop {
            if self.layout[curr_vertical_idx as usize][horizontal_idx as usize].is_clue() {
                break slot_count;
            } else if curr_vertical_idx == 0 {
                slot_count += 1;
                break slot_count;
            }

            slot_count += 1;
            curr_vertical_idx -= 1;
        };

        slot_count = -1;
        let horizontal_word_len = loop {
            if self.layout[vertical_idx as usize][curr_horizontal_idx as usize].is_clue() {
                break slot_count;
            } else if curr_horizontal_idx == 0 {
                slot_count += 1;
                break slot_count;
            }

            slot_count += 1;
            curr_horizontal_idx -= 1;
        };

        return max(vertical_word_len, horizontal_word_len);
    }

    fn assign_letter(curr_cell: &mut GridCell, mut assigned_states: BTreeSet<PossibleCellState>) -> BTreeSet<PossibleCellState> {
        curr_cell.assign_letter_state();
        assigned_states.insert(PossibleCellState::Letter);
        return assigned_states;
    }

    fn assign_clue_states(curr_cell: &mut GridCell, mut rng: ThreadRng, mut assigned_states: BTreeSet<PossibleCellState>, randomize: bool) -> BTreeSet<PossibleCellState> {

        let mut num_assigned_states = 0;
        let mut random_number: f32 = 0.0; // starts at 0.0 so at least one state is assigned

        while num_assigned_states < 2 && random_number < constants::THRESHOLD_PROBABILITY_FOR_SECOND_CLUE_STATE {
            match curr_cell.assign_clue_state_randomly(&mut rng, &assigned_states) {
                Some(state) => {
                    assigned_states.insert(state);
                    num_assigned_states += 1;
                    random_number = if randomize { rng.random() } else { 0.0 };
                }
                None => break,
            }
        }

        assigned_states
    }

    fn apply_first_row_column_restrictions(&mut self, vertical_idx: u32, horizontal_idx: u32) -> () {

        if vertical_idx == 0 && horizontal_idx > 0 && self.layout[vertical_idx as usize][(horizontal_idx - 1) as usize].is_letter() {
            self.layout[vertical_idx as usize][horizontal_idx as usize].force_clue();
        }
        else if vertical_idx == 0 && horizontal_idx > 0 && !self.layout[vertical_idx as usize][(horizontal_idx - 1) as usize].is_clue_of_type(SlotDirection::DownOnRightSide) {
            self.layout[vertical_idx as usize][horizontal_idx as usize].force_clue();
        }
        else if horizontal_idx == 0 && vertical_idx > 0 && self.layout[(vertical_idx - 1) as usize][horizontal_idx as usize].is_letter() {
            self.layout[vertical_idx as usize][horizontal_idx as usize].force_clue();
        }
        else if horizontal_idx == 0 && vertical_idx > 0 && !self.layout[(vertical_idx - 1) as usize][horizontal_idx as usize].is_clue_of_type(SlotDirection::RightOnBottomSide) {
            self.layout[vertical_idx as usize][horizontal_idx as usize].force_clue();
        }
    }

    fn restrict_neighborhood(&mut self, vertical_idx: u32, horizontal_idx: u32, assigned_cell_states: &BTreeSet<PossibleCellState>) -> Result<(), LayoutError> {

        if assigned_cell_states.iter().all(|state| matches!(state, PossibleCellState::Clue(_))) {

            if assigned_cell_states.iter().any(|state| matches!(state, PossibleCellState::Clue(SlotDirection::Right))) 
            {
                self.ensure_clear_slot(vertical_idx, horizontal_idx, SlotDirection::Right)?;
            }

            if assigned_cell_states.iter().any(|state| matches!(state, PossibleCellState::Clue(SlotDirection::RightOnBottomSide))) 
            {   
                self.ensure_clear_slot(vertical_idx, horizontal_idx, SlotDirection::RightOnBottomSide)?;
                self.ensure_wall_start(vertical_idx, horizontal_idx, SlotDirection::RightOnBottomSide)?;
            }

            if assigned_cell_states.iter().any(|state| matches!(state, PossibleCellState::Clue(SlotDirection::Down))) 
            {
                self.ensure_clear_slot(vertical_idx, horizontal_idx, SlotDirection::Down)?;
            }

            if assigned_cell_states.iter().any(|state| matches!(state, PossibleCellState::Clue(SlotDirection::DownOnRightSide))) 
            {
                self.ensure_clear_slot(vertical_idx, horizontal_idx, SlotDirection::DownOnRightSide)?;
                self.ensure_wall_start(vertical_idx, horizontal_idx, SlotDirection::DownOnRightSide)?;
            }
        }
        else if assigned_cell_states.iter().all(|state| matches!(state, PossibleCellState::Letter))
        {
            if self.is_start_of_angled_horizontal_slot(vertical_idx, horizontal_idx) {
                self.prevent_letter_underneath(vertical_idx, horizontal_idx);
            }

            if self.is_start_of_angled_vertical_slot(vertical_idx, horizontal_idx) {
                self.prevent_letter_on_the_right(vertical_idx, horizontal_idx);
            }
        }
        else {
            panic!("The cell {}, {} has assigned both a Clue state and a Letter state.", vertical_idx, horizontal_idx);
        }

        Ok(())
    }

    fn ensure_clear_slot(&mut self, vertical_idx: u32, horizontal_idx: u32, slot_direction: SlotDirection) -> Result<(), LayoutError> {

        match slot_direction {
            SlotDirection::Right => {
                self.check_straight_clear_slot(vertical_idx, horizontal_idx, 1, 0)?;
            },
            SlotDirection::Down => {
                self.check_straight_clear_slot(vertical_idx, horizontal_idx, 0, 1)?;
            },
            SlotDirection::RightOnBottomSide => {
                self.check_angled_clear_slot(vertical_idx, horizontal_idx, 0, 1)?;
            },
            SlotDirection::DownOnRightSide => {
                self.check_angled_clear_slot(vertical_idx, horizontal_idx, 1, 0)?;
            }
        }

        Ok(())
    }

    fn check_straight_clear_slot(&mut self, vertical_idx: u32, horizontal_idx: u32, dx: u32, dy: u32) -> Result<(), LayoutError> {
        
        let limit = if dx != 0 { self.width } else { self.height };
        let idx = if dx != 0 { horizontal_idx } else { vertical_idx };
        
        if idx + 2 >= limit
        {
            return Err(LayoutError::CannotEnsureAClearWordSlot);
        }

        self.layout[(vertical_idx + dy) as usize][(horizontal_idx + dx) as usize].force_letter();
        self.layout[(vertical_idx + dy * 2) as usize][(horizontal_idx + dx * 2) as usize].force_letter();

        Ok(())
    }

    fn check_angled_clear_slot(&mut self, vertical_idx: u32, horizontal_idx: u32, dx: u32, dy: u32) -> Result<(), LayoutError> 
    {
        if vertical_idx + 1 >= self.height || horizontal_idx + 1 >= self.width 
        {
            return Err(LayoutError::CannotEnsureAClearWordSlot);
        }

        self.layout[(vertical_idx + dy) as usize][(horizontal_idx + dx) as usize].force_letter();
        self.layout[(vertical_idx + 1) as usize][(horizontal_idx + 1) as usize].force_letter();

        Ok(())
    }

    fn ensure_wall_start(&mut self, vertical_idx: u32, horizontal_idx: u32, slot_direction: SlotDirection) -> Result<(), LayoutError> {

        let check_for_clear_slot = |primary_idx: u32, secondary_idx: u32| -> Result<(), LayoutError> {

            let limit = if primary_idx == vertical_idx { self.height } else { self.width };

            if primary_idx == limit - 1 {
                panic!("Impossible placement!")
            }

            if primary_idx >= limit - 2 {
                return Err(LayoutError::CannotEnsureAClearWordSlot);
            }

            if secondary_idx == 0 {
                return Ok(());
            } 
            else
            {
                panic!("Trying to place angled clue not on first column or row!");
            }
        };

        match slot_direction {
            SlotDirection::RightOnBottomSide => {
                check_for_clear_slot(vertical_idx, horizontal_idx)?;

                Ok(())
            },
            SlotDirection::DownOnRightSide => {
                check_for_clear_slot(horizontal_idx, vertical_idx)?;

                Ok(())
            },
            _ => panic!("Trying to ensure a corner start for a straight clue!")
        }
    }

    fn is_start_of_angled_horizontal_slot(&self, vertical_idx: u32, horizontal_idx: u32) -> bool {
        return vertical_idx > 0 && self.layout[(vertical_idx - 1) as usize][horizontal_idx as usize].is_clue_of_type(SlotDirection::RightOnBottomSide)
    }

    fn is_start_of_angled_vertical_slot(&self, vertical_idx: u32, horizontal_idx: u32) -> bool {
        return horizontal_idx > 0 && self.layout[vertical_idx as usize][(horizontal_idx - 1) as usize].is_clue_of_type(SlotDirection::DownOnRightSide)
    }

    fn prevent_letter_underneath(&mut self, vertical_idx: u32, horizontal_idx: u32) -> () {
        if vertical_idx == self.height - 1 {
            return;
        }

        self.layout[(vertical_idx + 1) as usize][horizontal_idx as usize].force_clue();
    }

    fn prevent_letter_on_the_right(&mut self, vertical_idx: u32, horizontal_idx: u32) -> () {
        if horizontal_idx == self.width - 1 {
            return;
        }

        self.layout[vertical_idx as usize][(horizontal_idx + 1) as usize].force_clue();
    }
}
