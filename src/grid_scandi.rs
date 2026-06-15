use std::cmp::max;
use std::{collections::{BTreeSet, HashMap}};
use rand::prelude::*;

use std::rc::Rc;
use std::cell::RefCell;

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
use crate::grid::LayoutError;
use crate::grid_scandi::gridcell::PossibleCellState;
use crate::grid::Grid;

static WORDS: &str = include_str!("../word_files/common_english_words.txt");

mod constants {
    pub const TARGET_CLUE_DENSITY : f32                        = 0.25;
    pub const THRESHOLD_PROBABILITY_FOR_SECOND_CLUE_STATE: f32 = 0.6;
    pub const _MAXIMUM_WORD_LENGTH : i32                       = 10; // unused currently
    pub const MEDIAN_WORD_LENGTH : i32                         = 6; 
    pub const PER_LETTER_LENGTH_PRESSURE : f32                 = 0.08;
    pub const _DEFICIT_RATE_DEFAULT : f32                      = 0.35; // unused currently
}

pub struct ScandiGrid {
    width: u32,
    height: u32,
    layout: Vec<Vec<Rc<RefCell<GridCell>>>>,
    clues_placed: u32,
    num_cells_accessed: u32,
    word_slots: Vec<Slot>
}

impl Grid for ScandiGrid {
    fn initialize(width: u32, height: u32) -> Self {
        let mut layout : Vec<Vec<Rc<RefCell<GridCell>>>> = Vec::new();

        for row_idx in 0..height {
            layout.push(Vec::new());

            for col_idx in 0..width {
                let cell = GridCell::build(row_idx, col_idx, width, height);

                layout[row_idx as usize].push(Rc::new(RefCell::new(cell)));
            }
        }
        
        let num_cells_accessed = 0;
        let clues_placed = 0;

        let word_slots = Vec::new();

        Self { width, height, layout, clues_placed, num_cells_accessed, word_slots }
    }

    fn construct(&mut self) -> Result<(), LayoutError> {

        let mut word_collections_per_length : HashMap<u32, Rc<RefCell<BTreeSet<String>>>> = HashMap::new();

        let words_vec : Vec<&str> = WORDS.lines().collect();
        for word in words_vec {
            word_collections_per_length
                .entry(word.len() as u32)
                .or_default()
                .borrow_mut()
                .insert(word.to_string());
        }

        let mut slot_id : u32 = 0;

        for vertical_idx in 0..self.height {
            for horizontal_idx in 0..self.width {

                let assigned_cell_states = self.set_cell_state_from_remaining_possibilities(vertical_idx, horizontal_idx);
                if let Err(_) = self.restrict_neighborhood(vertical_idx, horizontal_idx, &assigned_cell_states) {
                    // backtrack
                }

                let at_end_of_height = vertical_idx == self.height - 1;
                let at_end_of_width = horizontal_idx == self.width - 1;
                let at_end_of_grid = at_end_of_height || at_end_of_width;
                let on_first_row = vertical_idx == 0;
                let on_first_col = horizontal_idx == 0;
                let encountered_clue_cell = assigned_cell_states.iter().any(|state| matches!(state, PossibleCellState::Clue(_)));

                if  !on_first_row && !on_first_col && (encountered_clue_cell || at_end_of_grid) 
                {
                    if encountered_clue_cell || at_end_of_height {

                        let consider_last_cell = !encountered_clue_cell;

                        // println!("Finding vertical word len after hitting clue at {}, {}", vertical_idx, horizontal_idx);
                        // println!("Considering last cell? {}", consider_last_cell);
                        let (word_length_from_vertical, slot_direction_for_vertical, vertical_slot_cells) = self.find_vertical_word_len(vertical_idx, horizontal_idx, consider_last_cell);

                        // Only create a slot if the current clue is not adjacent to another clue
                        if word_length_from_vertical != 0 {

                            if !word_collections_per_length.contains_key(&(word_length_from_vertical as u32)) {
                                // TODO: backtrack here instead of panicking
                                panic!("1: Cannot fill slot at all!");
                            }

                            let available_words_for_vertical_slot = Rc::clone(&word_collections_per_length[&(word_length_from_vertical as u32)]);
                            // let adjusted_vertical_idx = if at_end_of_height { vertical_idx + 1 } else { vertical_idx }; 

                            let vertical_slot = match slot_direction_for_vertical {
                                SlotDirection::DownOnRightSide => 
                                    Slot::new(
                                        slot_id,
                                        // word_length_from_vertical as u32, 
                                        Rc::clone(&available_words_for_vertical_slot), 
                                        // Rc::clone(&(self.layout[(adjusted_vertical_idx - word_length_from_vertical as u32) as usize][(horizontal_idx - 1) as usize])), 
                                        vertical_slot_cells, 
                                        // slot_direction_for_vertical
                                    ),
                                SlotDirection::Down => 
                                    Slot::new(
                                        slot_id,
                                        // word_length_from_vertical as u32, 
                                        Rc::clone(&available_words_for_vertical_slot), 
                                        // Rc::clone(&(self.layout[(adjusted_vertical_idx - word_length_from_vertical as u32 - 1) as usize][horizontal_idx as usize])), 
                                        vertical_slot_cells, 
                                        // slot_direction_for_vertical
                                    ),
                                _ => panic!("Impossible clue cell placement.")
                            };

                            // println!("1: Pushing slot id {}", slot_id);
                            self.word_slots.push(vertical_slot);
                            slot_id += 1;
                        }

                    }

                    if encountered_clue_cell || at_end_of_width {

                        let consider_last_cell = !encountered_clue_cell;

                        // println!("Finding horizontal word len after hitting clue at {}, {}", vertical_idx, horizontal_idx);
                        // println!("Considering last cell? {}", consider_last_cell);
                        let (word_length_from_horizontal, slot_direction_for_horizontal, horizontal_slot_cells) = self.find_horizontal_word_len(vertical_idx, horizontal_idx, consider_last_cell);

                        if word_length_from_horizontal != 0 {
                            if !word_collections_per_length.contains_key(&(word_length_from_horizontal as u32)) {
                                // TODO: backtrack here instead of panicking
                                panic!("2: Cannot fill slot at all.");
                            }

                            let available_words_for_horizontal_slot = Rc::clone(&word_collections_per_length[&(word_length_from_horizontal as u32)]);
                            // let adjusted_horizontal_idx = if at_end_of_width { horizontal_idx + 1 } else { horizontal_idx }; 

                            let horizontal_slot = match slot_direction_for_horizontal {
                                SlotDirection::RightOnBottomSide => 
                                    Slot::new(
                                        slot_id,
                                        // word_length_from_horizontal as u32, 
                                        Rc::clone(&available_words_for_horizontal_slot), 
                                        // Rc::clone(&(self.layout[(vertical_idx - 1) as usize][(adjusted_horizontal_idx - word_length_from_horizontal as u32) as usize])), 
                                        horizontal_slot_cells, 
                                        // slot_direction_for_horizontal
                                    ),
                                SlotDirection::Right => 
                                    Slot::new(
                                        slot_id,
                                        // word_length_from_horizontal as u32, 
                                        Rc::clone(&available_words_for_horizontal_slot),
                                        // Rc::clone(&(self.layout[vertical_idx as usize][(adjusted_horizontal_idx - word_length_from_horizontal as u32 - 1) as usize])), 
                                        horizontal_slot_cells, 
                                        // slot_direction_for_horizontal
                                    ),
                                _ => panic!("Impossible clue cell placement.")
                            };

                            // println!("2: Pushing slot id {}", slot_id);
                            self.word_slots.push(horizontal_slot);
                            slot_id += 1;
                        }
                    }
                }
            }
        }

        // println!("Before sorting...");
        self.word_slots.sort_by(|slot1, slot2| slot1.get_suitable_word_set().len().cmp(&slot2.get_suitable_word_set().len()));
        // println!("After sorting...");

        for i in 0..self.word_slots.len() {

            // println!("in loop for {}", i);
            let slot_id = self.word_slots[i].get_slot_id();

            // println!("slot_id to try {}", slot_id);
            self.try_word_allocation(slot_id)?;
        }

        Ok(())
    } 

    fn print(&self) -> () {
        for vertical_idx in 0..self.height {
            for horizontal_idx in 0..self.width {
                self.layout[vertical_idx as usize][horizontal_idx as usize].borrow().print();
            }

            println!()
        }
    }
}

impl ScandiGrid {

    fn try_word_allocation(&mut self, slot_id: u32) -> Result<(), LayoutError> {

        // println!("Trying for slot id {}", slot_id);

        let mut already_recursed_crossing_ids = BTreeSet::new();
        while let Err((LayoutError::NoPossibleDomain, crossing_ids)) = 
        {
            // println!("Before getting slot.");
            let slot = self.get_slot(slot_id).unwrap();
            // println!("After getting slot.");
            slot.allocate_word()
        } 
        {
            if crossing_ids.is_empty()  {
                return Err(LayoutError::NoPossibleDomainAfterRecursion);
            }

            // println!("Before 1.");

            // 1. if crossing_ids is the same as already_recursed_crossing_ids, clear already_recursed_crossing_ids and reset to a full set of crossing ids.
            let mut slot_ids_to_backtrack : BTreeSet<&u32> = crossing_ids.difference(&already_recursed_crossing_ids).collect();
            if !crossing_ids.is_empty() && slot_ids_to_backtrack.is_empty() {
                slot_ids_to_backtrack = crossing_ids.iter().collect();
            }

            // println!("Before 2.");

            // 2. deallocate and discard the word in the slot of the last id in the slot_ids_to_backtrack 
            let backtrack_id = **(slot_ids_to_backtrack.last().unwrap());
            {
                let slot_to_backtrack = self.get_slot(backtrack_id).unwrap();
                slot_to_backtrack.deallocate_and_discard_word();
            }

            // println!("Before 3.");

            // 3. recursively call the same while let loop for that slot
            self.try_word_allocation(backtrack_id)?;

            // println!("Before 4.");

            // 4. assign the slot_id in the already_recursed_crossing_ids
            already_recursed_crossing_ids.insert(backtrack_id);

            // println!("Before end.");
        }

        Ok(())
    }

    fn get_slot(&mut self, slot_id: u32) -> Option<&mut Slot> {

        // println!("Getting slot for slot id {}", slot_id);
        for slot in &mut self.word_slots {

            let curr_slot_id = slot.get_slot_id();
            if curr_slot_id == slot_id {
                return Some(slot);
            }
        }

        None
    }

    fn set_cell_state_from_remaining_possibilities(&mut self, vertical_idx: u32, horizontal_idx: u32) -> BTreeSet<PossibleCellState> {

        self.apply_first_row_column_restrictions(vertical_idx, horizontal_idx);
        let current_word_len : i32 = self.word_len_up_to_curr_cell(vertical_idx, horizontal_idx);

        let mut curr_cell = self.layout[vertical_idx as usize][horizontal_idx as usize].borrow_mut();
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
                return Self::assign_clue_states(&mut *curr_cell, rng, assigned_states, false);
            } 
            else 
            {
                self.num_cells_accessed += 1;
                return Self::assign_letter(&mut *curr_cell, assigned_states); 
            }
        } 
        else if (horizontal_idx == 0 || vertical_idx == 0) && curr_cell.can_still_be_clue() 
        {
            // self.clues_placed += 1;
            return Self::assign_clue_states(&mut *curr_cell, rng, assigned_states, true);
        }
        else if curr_cell.can_still_be_clue() 
        {
            self.num_cells_accessed += 1;
            self.clues_placed += 1;
            return Self::assign_clue_states(&mut *curr_cell, rng, assigned_states, false);
        } 
        else if curr_cell.can_still_be_letter() 
        {
            self.num_cells_accessed += 1;
            return Self::assign_letter(&mut *curr_cell, assigned_states); 
        } 
        else 
        {
            panic!("Reached point of assigning state with no available possibilities.")
        }
    }

    fn find_vertical_word_len(&self, vertical_idx: u32, horizontal_idx: u32, end_of_grid: bool) -> (i32, SlotDirection, Vec<Rc<RefCell<GridCell>>>) {

        let mut slot_count = if end_of_grid { 0 } else { -1 };
        let mut curr_vertical_idx = vertical_idx;

        let mut slot_cells : Vec<Rc<RefCell<GridCell>>> = Vec::new();

        let (vertical_word_len, slot_direction) = loop {
            if curr_vertical_idx != vertical_idx && self.layout[curr_vertical_idx as usize][horizontal_idx as usize].borrow().is_clue() {
                break (slot_count, SlotDirection::Down);
            } else if curr_vertical_idx == 0 {
                slot_count += 1;
                slot_cells.push(Rc::clone(&self.layout[curr_vertical_idx as usize][horizontal_idx as usize]));
                break (slot_count, SlotDirection::DownOnRightSide);
            }

            if slot_count >= 0 {
                slot_cells.push(Rc::clone(&self.layout[curr_vertical_idx as usize][horizontal_idx as usize]));
            }

            slot_count += 1;
            curr_vertical_idx -= 1;
        };

        slot_cells.reverse();

        // println!("Len is {}", vertical_word_len);
        return (vertical_word_len, slot_direction, slot_cells);
    }

    fn find_horizontal_word_len(&self, vertical_idx: u32, horizontal_idx: u32, end_of_grid: bool) -> (i32, SlotDirection, Vec<Rc<RefCell<GridCell>>>) {
        let mut slot_count = if end_of_grid { 0 } else { -1 };
        let mut curr_horizontal_idx = horizontal_idx;

        let mut slot_cells : Vec<Rc<RefCell<GridCell>>> = Vec::new();

        let (horizontal_word_len, slot_direction) = loop {
            if curr_horizontal_idx != horizontal_idx && self.layout[vertical_idx as usize][curr_horizontal_idx as usize].borrow().is_clue() {
                break (slot_count, SlotDirection::Right);
            } else if curr_horizontal_idx == 0 {
                slot_count += 1;
                slot_cells.push(Rc::clone(&self.layout[vertical_idx as usize][curr_horizontal_idx as usize]));
                break (slot_count, SlotDirection::RightOnBottomSide);
            }

            if slot_count >= 0 {
                slot_cells.push(Rc::clone(&self.layout[vertical_idx as usize][curr_horizontal_idx as usize]));
            }

            slot_count += 1;
            curr_horizontal_idx -= 1;
        };

        slot_cells.reverse();

        return (horizontal_word_len, slot_direction, slot_cells);
    }
    

    fn word_len_up_to_curr_cell(&self, vertical_idx: u32, horizontal_idx: u32) -> i32 {

        let (vertical_word_len, _, _) = self.find_vertical_word_len(vertical_idx, horizontal_idx, false);
        let (horizontal_word_len, _, _) = self.find_horizontal_word_len(vertical_idx, horizontal_idx, false);

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

        if vertical_idx == 0 && horizontal_idx > 0 && self.layout[vertical_idx as usize][(horizontal_idx - 1) as usize].borrow().is_letter() {
            self.layout[vertical_idx as usize][horizontal_idx as usize].borrow_mut().force_clue();
        }
        else if vertical_idx == 0 && horizontal_idx > 0 && !self.layout[vertical_idx as usize][(horizontal_idx - 1) as usize].borrow().is_clue_of_type(SlotDirection::DownOnRightSide) {
            self.layout[vertical_idx as usize][horizontal_idx as usize].borrow_mut().force_clue();
        }
        else if horizontal_idx == 0 && vertical_idx > 0 && self.layout[(vertical_idx - 1) as usize][horizontal_idx as usize].borrow().is_letter() {
            self.layout[vertical_idx as usize][horizontal_idx as usize].borrow_mut().force_clue();
        }
        else if horizontal_idx == 0 && vertical_idx > 0 && !self.layout[(vertical_idx - 1) as usize][horizontal_idx as usize].borrow().is_clue_of_type(SlotDirection::RightOnBottomSide) {
            self.layout[vertical_idx as usize][horizontal_idx as usize].borrow_mut().force_clue();
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

        self.layout[(vertical_idx + dy) as usize][(horizontal_idx + dx) as usize].borrow_mut().force_letter();
        self.layout[(vertical_idx + dy * 2) as usize][(horizontal_idx + dx * 2) as usize].borrow_mut().force_letter();

        Ok(())
    }

    fn check_angled_clear_slot(&mut self, vertical_idx: u32, horizontal_idx: u32, dx: u32, dy: u32) -> Result<(), LayoutError> 
    {
        if vertical_idx + 1 >= self.height || horizontal_idx + 1 >= self.width 
        {
            return Err(LayoutError::CannotEnsureAClearWordSlot);
        }

        self.layout[(vertical_idx + dy) as usize][(horizontal_idx + dx) as usize].borrow_mut().force_letter();
        self.layout[(vertical_idx + 1) as usize][(horizontal_idx + 1) as usize].borrow_mut().force_letter();

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
        return vertical_idx > 0 && self.layout[(vertical_idx - 1) as usize][horizontal_idx as usize].borrow().is_clue_of_type(SlotDirection::RightOnBottomSide)
    }

    fn is_start_of_angled_vertical_slot(&self, vertical_idx: u32, horizontal_idx: u32) -> bool {
        return horizontal_idx > 0 && self.layout[vertical_idx as usize][(horizontal_idx - 1) as usize].borrow().is_clue_of_type(SlotDirection::DownOnRightSide)
    }

    fn prevent_letter_underneath(&mut self, vertical_idx: u32, horizontal_idx: u32) -> () {
        if vertical_idx == self.height - 1 {
            return;
        }

        self.layout[(vertical_idx + 1) as usize][horizontal_idx as usize].borrow_mut().force_clue();
    }

    fn prevent_letter_on_the_right(&mut self, vertical_idx: u32, horizontal_idx: u32) -> () {
        if horizontal_idx == self.width - 1 {
            return;
        }

        self.layout[vertical_idx as usize][(horizontal_idx + 1) as usize].borrow_mut().force_clue();
    }
}
