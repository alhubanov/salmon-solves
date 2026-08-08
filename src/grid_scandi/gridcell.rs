use rand::prelude::*;
use serde::Serialize;
use super::slot::SlotDirection;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Serialize)]
pub enum CellType {
    Clue(Vec<(String, u32, SlotDirection)>),
    Letter(Option<char>, Vec<u32>)
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash, Serialize)]
pub enum PossibleCellState {
    Letter,
    Clue(SlotDirection)
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct GridCell {
    pub cell: Option<CellType>,
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
                Some(CellType::Letter(letter, _)) => print!("{}", letter.unwrap()),
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

    pub fn get_foreign_slot_id_from_letter_cell(&self, slot_id: u32) -> Option<&u32> 
    {
        match &self.cell 
        {
            Some(CellType::Letter(_, recorded_ids)) => 
            { 
                return recorded_ids.iter().find(|elem| *elem != &slot_id);
            },
            _ => panic!("Retrieving slot id record to non-letter cell.")
        };
    }

    pub fn contains_slot_id(&self, slot_id: u32) -> bool
    {
        match &self.cell 
        {
            None => return false,
            Some(CellType::Letter(_, recorded_ids)) => 
            { 
                return recorded_ids.contains(&slot_id); 
            },
            _ => panic!("Checking if slot id record is contained in a non-letter cell.")
        };
    }

    pub fn add_slot_id_to_letter_cell(&mut self, slot_id: Option<u32>) -> () {
        if let None = slot_id {
            return;
        }

        match &mut self.cell 
        {
            None => 
            {
                let mut new_vec : Vec<u32> = Vec::new();
                new_vec.push(slot_id.unwrap());
                self.cell = Some(CellType::Letter(None, new_vec));
            },
            Some(CellType::Letter(_, recorded_ids)) => 
            { 
                recorded_ids.push(slot_id.unwrap()); 
            },
            _ => panic!("Adding slot id record to non-letter cell.")
        };
    }

    pub fn append_clue(&mut self, slot_id: u32, slot_direction: SlotDirection, clue: String) -> ()
    {
        match &mut self.cell
        {
            Some(CellType::Clue(clue_vec)) => 
            {
                clue_vec.push((clue, slot_id, slot_direction));
            },
            Some(CellType::Letter(_, _)) => panic!("Associated clue cell is a letter."),
            None => panic!("Already exhausted possibility.")
        }
    }

    pub fn has_no_letter_assigned(&self) -> bool
    {
        match &self.cell
        {
            // TODO: avoid cloning the vector here
            Some(CellType::Letter(None, _)) => { return true; },
            None => { return true; }
            _ => { return false; }
        }
    }

    pub fn assign_letter(&mut self, letter: char) -> ()
    {
        match &mut self.cell
        {
            // TODO: avoid cloning the vector here
            Some(CellType::Letter(_, recorded_ids)) => { self.cell = Some(CellType::Letter(Some(letter), recorded_ids.clone())) },
            None => { self.cell = Some(CellType::Letter(Some(letter), Vec::new())) }
            _ => panic!("Assinging letter to a non-letter cell.")
        }
    }

    pub fn discard_letter(&mut self) -> ()
    {
        match &mut self.cell
        {
            // TODO: avoid cloning the vector here
            Some(CellType::Letter(_, recorded_ids)) => { self.cell = Some(CellType::Letter(None, recorded_ids.clone())) },
            None => { return; }
            _ => panic!("Discarding letter from a non-letter cell.")
        }
    }

    pub fn get_slot_ids_owned_by_crossing_slot<F>(&self, slot_id: u32, crossing_slot_has_word: &mut F) -> bool
    where 
        F: Fn(u32) -> bool,   
    {
        match &self.cell
        {
            Some(CellType::Letter(_, recorded_ids)) => 
            { 
                return recorded_ids.iter().any(|&id| id != slot_id && crossing_slot_has_word(id));
            },
            None => panic!("Retrieving slot ids from an empty cell."),
            _ => panic!("Retrieving slot ids from a non-letter cell when a letter cell is expected")
        }
    }
}
