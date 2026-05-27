use serde::{Serialize, Deserialize};
use std::collections::{BTreeSet};
use rand::prelude::*;

use super::clue::{Clue, SlotDirection};
use super::letter::Letter;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub enum CellType {
    Clue(Clue),
    Letter(Letter)
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Clone, Copy, Debug)]
pub enum PossibleCellState {
    Letter,
    Clue(SlotDirection)
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GridCell {
    cell: Option<CellType>,
    assigned_cell_states: BTreeSet<PossibleCellState>,
    possible_remaining_cell_states: BTreeSet<PossibleCellState>
}

impl GridCell {

    pub fn build(row_idx: u32, col_idx: u32, width: u32, height: u32) -> Self {
        let mut cell_states = BTreeSet::new();

        if row_idx == 0 && col_idx == 0 {
            cell_states.insert(PossibleCellState::Clue(SlotDirection::DownOnRightSide)); 
            cell_states.insert(PossibleCellState::Clue(SlotDirection::RightOnBottomSide));
        } else if row_idx == 0 {
            // cell_states.insert(PossibleCellState::Letter);
            cell_states.insert(PossibleCellState::Clue(SlotDirection::Down));

            if col_idx < width - 1 {
                cell_states.insert(PossibleCellState::Clue(SlotDirection::DownOnRightSide));
            } 
        } else if col_idx == 0 {
            // cell_states.insert(PossibleCellState::Letter);
            cell_states.insert(PossibleCellState::Clue(SlotDirection::Right));

            if row_idx < height - 1 {
                cell_states.insert(PossibleCellState::Clue(SlotDirection::RightOnBottomSide));
            }
        } else {
            cell_states.insert(PossibleCellState::Letter);

            if row_idx < height - 2 {
                cell_states.insert(PossibleCellState::Clue(SlotDirection::Down));
            }

            if col_idx < width - 2 {
                cell_states.insert(PossibleCellState::Clue(SlotDirection::Right));
            }
        }

        Self { 
            cell: None,
            assigned_cell_states: BTreeSet::new(),
            possible_remaining_cell_states: cell_states
        }
    }

    pub fn print(&self) -> () { 
        if self.assigned_cell_states.contains(&PossibleCellState::Clue(SlotDirection::Down)) {
            print!("|");
        }
        
        if self.assigned_cell_states.contains(&PossibleCellState::Clue(SlotDirection::DownOnRightSide)) {
            print!("]");
        }
        
        if self.assigned_cell_states.contains(&PossibleCellState::Clue(SlotDirection::Right)) {
            print!("-");
        }
        
        if self.assigned_cell_states.contains(&PossibleCellState::Clue(SlotDirection::RightOnBottomSide)) {
            print!("[");
        }
        
        if self.assigned_cell_states.iter().any(|state| matches!(state, PossibleCellState::Letter)) {
            print!("_");
        }

        print!(" ");
    }

    pub fn assign_clue_state_randomly(&mut self, rng: &mut ThreadRng, previously_assigned_states: &BTreeSet<PossibleCellState>) -> Option<PossibleCellState> {

        // remove states that cannot be currently assigned
        let letter_state_removed = self.possible_remaining_cell_states.remove(&PossibleCellState::Letter);
        for assigned_state in previously_assigned_states {
            self.possible_remaining_cell_states.remove(&assigned_state);
        }

        // assign a state randomly
        let sampled_state = self.possible_remaining_cell_states
                                .iter()
                                .choose(rng)
                                .cloned();

        if let None = sampled_state {
            return None;
        } 

        self.assigned_cell_states.insert(sampled_state.unwrap());

        // insert back all removed states
        if letter_state_removed {
            self.possible_remaining_cell_states.insert(PossibleCellState::Letter);
        }

        for assigned_state in previously_assigned_states {
            self.possible_remaining_cell_states.insert(*assigned_state);
        }

        return Some(sampled_state.unwrap()); 
    }

    pub fn assign_letter_state(&mut self) -> () {
        self.assigned_cell_states.insert(PossibleCellState::Letter);
    }

    pub fn force_letter(&mut self) -> () {
        self.possible_remaining_cell_states.clear();
        self.possible_remaining_cell_states.insert(PossibleCellState::Letter);
    }

    pub fn force_clue(&mut self) -> () {
        self.possible_remaining_cell_states.remove(&PossibleCellState::Letter);
    }

    pub fn can_still_be_letter(&self) -> bool {
        self.possible_remaining_cell_states.contains(&PossibleCellState::Letter)
    }

    pub fn can_still_be_clue(&self) -> bool {
        self.possible_remaining_cell_states
            .iter()
            .any(|state| matches!(state, PossibleCellState::Clue(_)))
    }

    pub fn is_clue(&self) -> bool {
        self.assigned_cell_states
            .iter()
            .any(|state| matches!(state, PossibleCellState::Clue(_)))
    }

    pub fn is_clue_of_type(&self, clue_kind: SlotDirection) -> bool {
        return self.assigned_cell_states.contains(&PossibleCellState::Clue(clue_kind));
    }
}
