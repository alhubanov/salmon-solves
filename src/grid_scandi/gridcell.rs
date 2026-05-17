use serde::{Serialize, Deserialize};
use std::collections::{BTreeSet};

use super::clue::{Clue, SlotDirection};
use super::letter::Letter;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub enum CellType {
    Clue(Clue),
    Letter(Letter)
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PossibleCellState {
    Letter,
    Clue(SlotDirection)
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GridCell {
    cell: Option<CellType>,
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
            possible_remaining_cell_states: cell_states
        }
    }

    pub fn build_starting_cell() -> Self {
        let mut cell_states = BTreeSet::new();
        cell_states.insert(PossibleCellState::Clue(SlotDirection::DownOnRightSide)); 
        cell_states.insert(PossibleCellState::Clue(SlotDirection::RightOnBottomSide));  

        Self { 
            cell: None, 
            possible_remaining_cell_states: cell_states
        }
    }

    pub fn reset(&mut self) {
        *self = Self::build_generic_cell();
    }

    pub fn is_undetermined(&self) -> bool {
        return self.cell.is_none();
    }
}
