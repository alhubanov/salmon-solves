use serde::{Serialize, Deserialize};
use std::collections::{BTreeSet};
use wasm_bindgen::prelude::*;
use rand::prelude::*;

mod clue;
mod letter;
mod gridcell;

use gridcell::GridCell;

use crate::grid_scandi::{clue::SlotDirection, gridcell::PossibleCellState};

pub enum LayoutError {
    NoPossibleDomain,
    LowClueDensity,
    HighClueDensity,
    IsolatedLetter,
    CannotEnsureAClearWordSlot
}

#[derive(Serialize, Deserialize)]
#[wasm_bindgen(js_name = GridScandi)]
pub struct Grid {
    width: u32,
    height: u32,
    layout: Vec<Vec<GridCell>>,
    placed_words: BTreeSet<String>,
    clue_density: f32,
    target_clue_density: f32
}

impl Grid {
    pub fn new(width: u32, height: u32) -> Self {
        let mut layout : Vec<Vec<GridCell>> = Vec::new();

        for row_idx in 0..height {
            layout.push(Vec::new());

            for col_idx in 0..width {
                let cell = GridCell::build(row_idx, col_idx, width, height);

                layout[row_idx as usize].push(cell);
            }
        }

        let placed_words = BTreeSet::new();
        let clue_density = 0.0;
        let target_clue_density = 0.25;

        Grid { width, height, layout, placed_words, clue_density, target_clue_density }
    }

    pub fn print(&self) {
        for vertical_idx in 0..self.height {
            for horizontal_idx in 0..self.width {
                self.layout[vertical_idx as usize][horizontal_idx as usize].print();
            }

            println!()
        }
    }

    pub fn construct_layout(&mut self) -> Result<(), LayoutError> {

        let mut num_clues = 0;

        for vertical_idx in 0..self.height {
            for horizontal_idx in 0..self.width {

                let assigned_cell_states = self.set_cell_state_from_remaining_possibilities(vertical_idx, horizontal_idx);
                if assigned_cell_states.iter().any(|state| matches!(state, PossibleCellState::Clue(_))) {
                    num_clues += 1;
                }

                if let Err(_) = self.check_neighborhood(vertical_idx, horizontal_idx, &assigned_cell_states) {
                    // backtrack
                }

                self.check_density(num_clues)?;
            }
        }

        Ok(())
    }

    fn set_cell_state_from_remaining_possibilities(&mut self, vertical_idx: u32, horizontal_idx: u32) -> BTreeSet<PossibleCellState> {

        let curr_cell = &mut self.layout[vertical_idx as usize][horizontal_idx as usize];

        let mut rng = rand::rng();
        let mut assigned_states : BTreeSet<PossibleCellState> = BTreeSet::new();

        if curr_cell.can_still_be_clue() && curr_cell.can_still_be_letter() 
        {
            let random_num = rng.random_range(0..100);
            if random_num as f32 <= (self.target_clue_density * 100.0) 
            {
                let mut num_assigned_states = 0;
                while num_assigned_states < 2 {
                    match curr_cell.assign_clue_state_randomly(&mut rng, &assigned_states) {
                        Some(state) => {
                            assigned_states.insert(state);
                            num_assigned_states += 1;
                        },
                        None => break
                    }
                }

                return assigned_states;
            } 
            else 
            {
                curr_cell.assign_letter_state();
                assigned_states.insert(PossibleCellState::Letter);
                return assigned_states;
            }

        } 
        else if curr_cell.can_still_be_clue() 
        {
            let mut num_assigned_states = 0;
            while num_assigned_states < 2 {
                match curr_cell.assign_clue_state_randomly(&mut rng, &assigned_states) {
                    Some(state) => {
                        assigned_states.insert(state);
                        num_assigned_states += 1;
                    },
                    None => break
                }
            }

            return assigned_states;
        } 
        else if curr_cell.can_still_be_letter() 
        {
            curr_cell.assign_letter_state();
            assigned_states.insert(PossibleCellState::Letter);
            return assigned_states;
        } 
        else 
        {
            panic!("Reached point of assigning state with no available possibilities.")
        }
    }

    fn check_neighborhood(&mut self, vertical_idx: u32, horizontal_idx: u32, assigned_cell_states: &BTreeSet<PossibleCellState>) -> Result<(), LayoutError> {

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
        if horizontal_idx == self.height - 1 {
            return;
        }

        self.layout[vertical_idx as usize][(horizontal_idx + 1) as usize].force_clue();
    }

    fn check_density(&self, num_clues: u32) -> Result<(), LayoutError> {
        Ok(())
    }
}
