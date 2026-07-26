use rand::prelude::*;
use serde::Serialize;

use super::clue::Clue;
use super::slot::SlotDirection;
use super::letter::Letter;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Serialize)]
pub enum CellType {
    Clue(Clue),
    Letter(Letter)
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash, Serialize)]
pub enum PossibleCellState {
    Letter,
    Clue(SlotDirection)
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct GridCell {
    pub cell: Option<CellType>,
    pub slot_ids: Option<Vec<u32>>,
    pub assigned_cell_states: Vec<PossibleCellState>,
    pub possible_remaining_cell_states: Vec<PossibleCellState>
}

impl GridCell {

    pub fn build(row_idx: u32, col_idx: u32, width: u32, height: u32) -> Self {
        let mut cell_states = Vec::new();

        if row_idx == 0 && col_idx == 0 {
            cell_states.push(PossibleCellState::Clue(SlotDirection::DownOnRightSide)); 
            cell_states.push(PossibleCellState::Clue(SlotDirection::RightOnBottomSide));
        } else if row_idx == 0 {
            cell_states.push(PossibleCellState::Letter);
            cell_states.push(PossibleCellState::Clue(SlotDirection::Down));

            if col_idx < width - 1 {
                cell_states.push(PossibleCellState::Clue(SlotDirection::DownOnRightSide));
            } 
        } else if col_idx == 0 {
            cell_states.push(PossibleCellState::Letter);
            cell_states.push(PossibleCellState::Clue(SlotDirection::Right));

            if row_idx < height - 1 {
                cell_states.push(PossibleCellState::Clue(SlotDirection::RightOnBottomSide));
            }
        } else {
            cell_states.push(PossibleCellState::Letter);

            if row_idx < height - 2 {
                cell_states.push(PossibleCellState::Clue(SlotDirection::Down));
            }

            if col_idx < width - 2 {
                cell_states.push(PossibleCellState::Clue(SlotDirection::Right));
            }
        }

        Self { 
            cell: None,
            slot_ids: None,
            assigned_cell_states: Vec::new(),
            possible_remaining_cell_states: cell_states
        }
    }

    pub fn print(&self) -> () 
    { 
        let mut count_printed = 0;
        if self.assigned_cell_states.contains(&PossibleCellState::Clue(SlotDirection::Down)) {
            print!("|");
            count_printed += 1;
        }
        
        if self.assigned_cell_states.contains(&PossibleCellState::Clue(SlotDirection::DownOnRightSide)) {
            print!("]");
            count_printed += 1;
        }
        
        if self.assigned_cell_states.contains(&PossibleCellState::Clue(SlotDirection::Right)) {
            print!("-");
            count_printed += 1;
        }
        
        if self.assigned_cell_states.contains(&PossibleCellState::Clue(SlotDirection::RightOnBottomSide)) {
            print!("[");
            count_printed += 1;
        }
        
        if self.assigned_cell_states.iter().any(|state| matches!(state, PossibleCellState::Letter)) {
            match &self.cell {
                None => print!("_"),
                Some(CellType::Letter(letter)) => print!("{}", letter.get_cell_value()),
                _ => print!("0 ")
            }

            count_printed += 1;
        }

        print!(" ");
        if count_printed == 1 {
            print!(" ");
        }
    }

    pub fn assign_clue_state(
        &mut self, 
        rng: &mut dyn Rng, 
        previously_assigned_states: &Vec<PossibleCellState>, 
        is_first_row: bool, 
        is_first_col: bool
    ) -> Option<PossibleCellState> 
    {
        // remove states that cannot be currently assigned
        let letter_state_removed = self.possible_remaining_cell_states.contains(&PossibleCellState::Letter);

        self.possible_remaining_cell_states.retain(|&state| state != PossibleCellState::Letter);
        for assigned_state in previously_assigned_states {
            self.possible_remaining_cell_states.retain(|state| state != assigned_state);
        }

        let is_top_left = is_first_row && is_first_col;

        // assign a state randomly but ensure no "non-slots" arise
        let sampled_state = if self.assigned_cell_states.is_empty() && !is_top_left && is_first_row && self.possible_remaining_cell_states.contains(&PossibleCellState::Clue(SlotDirection::Down))
            {   
                Some(PossibleCellState::Clue(SlotDirection::Down))
            } 
            else if self.assigned_cell_states.is_empty() && !is_top_left && is_first_col && self.possible_remaining_cell_states.contains(&PossibleCellState::Clue(SlotDirection::Right))
            {
                Some(PossibleCellState::Clue(SlotDirection::Right))
            } 
            else {
                self.possible_remaining_cell_states.iter().choose(rng).cloned()
            };

        if let None = sampled_state {
            return None;
        } 

        self.assigned_cell_states.push(sampled_state.unwrap());

        // insert back all removed states
        if letter_state_removed {
            self.possible_remaining_cell_states.push(PossibleCellState::Letter);
        }

        for assigned_state in previously_assigned_states {
            self.possible_remaining_cell_states.push(*assigned_state);
        }

        return Some(sampled_state.unwrap()); 
    }

    pub fn assign_letter_state(&mut self) -> () {
        self.assigned_cell_states.push(PossibleCellState::Letter);
    }

    pub fn force_letter(&mut self) -> () {
        self.possible_remaining_cell_states.clear();
        self.possible_remaining_cell_states.push(PossibleCellState::Letter);
    }

    pub fn force_clue(&mut self) -> () {
        self.possible_remaining_cell_states.retain(|&state| state != PossibleCellState::Letter);
    }

    pub fn can_still_be_letter(&self) -> bool {
        self.possible_remaining_cell_states.contains(&PossibleCellState::Letter)
    }

    pub fn can_still_be_clue(&self) -> bool {
        self.possible_remaining_cell_states
            .iter()
            .any(|state| matches!(state, PossibleCellState::Clue(_)))
    }

    pub fn is_letter(&self) -> bool {
        self.assigned_cell_states
            .iter()
            .any(|state| matches!(state, PossibleCellState::Letter))
    }

    pub fn is_clue(&self) -> bool {
        self.assigned_cell_states
            .iter()
            .any(|state| matches!(state, PossibleCellState::Clue(_)))
    }

    pub fn is_clue_of_type(&self, clue_kind: SlotDirection) -> bool {
        return self.assigned_cell_states.contains(&PossibleCellState::Clue(clue_kind));
    }

    pub fn add_slot_id(&mut self, slot_id: Option<u32>) -> () {
        if let None = slot_id {
            return;
        }

        match &mut self.slot_ids {
            Some(vector) => { vector.push(slot_id.unwrap()); },
            None => {
                let mut new_vec : Vec<u32> = Vec::new();
                new_vec.push(slot_id.unwrap());
                self.slot_ids = Some(new_vec);
            }
        };
    }
}
