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

#[derive(PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Copy, Clone)]
pub enum PossibleCellState {
    Letter,
    Clue(SlotDirection)
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GridCell {
    cell: Option<CellType>,
    assigned_cell_state: Option<PossibleCellState>,
    possible_remaining_cell_states: BTreeSet<PossibleCellState>
}

impl GridCell {
    pub fn build_generic_cell() -> Self {
        let mut cell_states = BTreeSet::new();
        cell_states.insert(PossibleCellState::Letter);
        cell_states.insert(PossibleCellState::Clue(SlotDirection::Down));
        cell_states.insert(PossibleCellState::Clue(SlotDirection::DownOnRightSide)); 
        cell_states.insert(PossibleCellState::Clue(SlotDirection::Right)); 
        cell_states.insert(PossibleCellState::Clue(SlotDirection::RightOnBottomSide));  

        Self { 
            cell: None,
            assigned_cell_state: None,
            possible_remaining_cell_states: cell_states
        }
    }

    pub fn build_starting_cell() -> Self {
        let mut cell_states = BTreeSet::new();
        cell_states.insert(PossibleCellState::Clue(SlotDirection::DownOnRightSide)); 
        cell_states.insert(PossibleCellState::Clue(SlotDirection::RightOnBottomSide));  

        Self { 
            cell: None, 
            assigned_cell_state: None,
            possible_remaining_cell_states: cell_states
        }
    }

    pub fn reset(&mut self) {
        *self = Self::build_generic_cell();
    }

    pub fn is_undetermined(&self) -> bool {
        return self.cell.is_none();
    }

    pub fn can_still_be_letter(&self) -> bool {
        self.possible_remaining_cell_states.contains(&PossibleCellState::Letter)
    }

    pub fn can_still_be_clue(&self) -> bool {
        self.possible_remaining_cell_states
            .iter()
            .any(|state| matches!(state, PossibleCellState::Clue(_)))
    }

    pub fn assign_clue_state_randomly(&mut self) -> PossibleCellState {

        let letter_state_removed = self.possible_remaining_cell_states.remove(&PossibleCellState::Letter);

        let mut rng = rand::rng();
        let sample_state = *self.possible_remaining_cell_states
                                .iter()
                                .choose(&mut rng)
                                .expect("Never try to assign clue state randomly from an empty set!");

        self.assigned_cell_state = Some(sample_state);

        if letter_state_removed {
            self.possible_remaining_cell_states.insert(PossibleCellState::Letter);
        }

        return sample_state; 
    }

    pub fn assign_letter_state(&mut self) -> () {
        self.assigned_cell_state = Some(PossibleCellState::Letter);
    }
}
