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
    words_placed: BTreeSet<String>,
    num_cells_accessed: u32,
    word_slots: Vec<Slot>
}

impl Grid for ScandiGrid {
    fn initialize(width: u32, height: u32) -> Self 
    {
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
        let words_placed = BTreeSet::new();
        let word_slots = Vec::new();

        Self { width, height, layout, clues_placed, words_placed, num_cells_accessed, word_slots }
    }

    fn construct(&mut self) -> Result<(), LayoutError> 
    {
        let mut word_collections_per_length : HashMap<u32, Rc<RefCell<BTreeSet<String>>>> = HashMap::new();

        let words_vec : Vec<&str> = WORDS.lines().collect();
        for word in words_vec 
        {
            word_collections_per_length
                .entry(word.len() as u32)
                .or_default()
                .borrow_mut()
                .insert(word.to_string());
        }

        self.create_all_slots(&word_collections_per_length);
        self.fill_grid()?;

        Ok(())
    } 

    fn print(&self) -> () 
    {
        for vertical_idx in 0..self.height 
        {
            for horizontal_idx in 0..self.width 
            {
                self.layout[vertical_idx as usize][horizontal_idx as usize].borrow().print();
            }

            println!()
        }
    }
}

impl ScandiGrid {

    fn get_slot(&mut self, slot_id: u32) -> Option<&mut Slot> 
    {
        for slot in &mut self.word_slots {

            let curr_slot_id = slot.get_slot_id();
            if curr_slot_id == slot_id {
                return Some(slot);
            }
        }

        None
    }

    fn get_len(&self, vertical_idx: u32, horizontal_idx: u32, end_of_grid: bool, orientation: &str) -> Result<i32, LayoutError> 
    {
        let mut slot_count = if end_of_grid { 0 } else { -1 };
        let mut curr_vertical_idx = vertical_idx;
        let mut curr_horizontal_idx = horizontal_idx;

        let clue_direction = if orientation == "vertical" { SlotDirection::Down } else { SlotDirection::Right };

        let word_len = loop 
        {
            let (active_orientation_idx, baseline_orientation_idx) = 
                if orientation == "vertical"
                {
                    (curr_vertical_idx, vertical_idx)
                }
                else // orientation == "horizontal" 
                {
                    (curr_horizontal_idx, horizontal_idx)
                };
            
            if active_orientation_idx != baseline_orientation_idx && self.layout[curr_vertical_idx as usize][curr_horizontal_idx as usize].borrow().is_clue()
            {
                if !self.layout[curr_vertical_idx as usize][curr_horizontal_idx as usize].borrow().is_clue_of_type(clue_direction) 
                {
                    return Err(LayoutError::IsNotAValidSlot);
                }
                break slot_count;
            }
            else if active_orientation_idx == 0 
            {
                slot_count += 1;
                break slot_count; 
            }

            slot_count += 1;
            if orientation == "vertical" { curr_vertical_idx -= 1; } else { curr_horizontal_idx -= 1; }
        };

        return Ok(word_len);
    }

    fn get_max_len(&self, vertical_idx: u32, horizontal_idx: u32) -> i32 
    {
        let vertical_word_len = self.get_len(vertical_idx, horizontal_idx, false, "vertical").unwrap_or(0);
        let horizontal_word_len = self.get_len(vertical_idx, horizontal_idx, false, "horizontal").unwrap_or(0);

        return max(vertical_word_len, horizontal_word_len);
    }

    fn get_slot_attributes(&self, mut vertical_idx: u32, mut horizontal_idx: u32, end_of_grid: bool, slot_id: Option<u32>, orientation: &str) -> Result<(i32, Vec<Rc<RefCell<GridCell>>>), LayoutError> 
    {
        let start_position = if end_of_grid { 0 } else { -1 };

        let mut slot_cells : Vec<Rc<RefCell<GridCell>>> = Vec::new();
        let word_len = self.get_len(vertical_idx, horizontal_idx, end_of_grid, orientation)?;

        for idx in start_position..word_len 
        {
            if idx == -1 
            {
                if orientation == "vertical" { vertical_idx -= 1; } else { horizontal_idx -= 1; } 
                continue; 
            }

            self.layout[vertical_idx as usize][horizontal_idx as usize].borrow_mut().insert_slot_id(slot_id);
            slot_cells.push(Rc::clone(&self.layout[vertical_idx as usize][horizontal_idx as usize]));  

            if idx < word_len - 1 {
                if orientation == "vertical" { vertical_idx -= 1; } else { horizontal_idx -= 1; } 
            }
        }

        slot_cells.reverse();
        return Ok((word_len, slot_cells));
    }

    fn create_slot(&mut self, 
                    vertical_idx: u32, 
                    horizontal_idx: u32, 
                    slot_id: &mut u32, 
                    word_collections_per_length : &HashMap<u32, Rc<RefCell<BTreeSet<String>>>>,
                    encountered_clue_cell: bool,
                    orientation: &str)  
    {
        let consider_last_cell = !encountered_clue_cell;
        let slot_attributes = self.get_slot_attributes(vertical_idx, horizontal_idx, consider_last_cell, Some(*slot_id), orientation);

        let (word_length, slot_cells) = match slot_attributes 
        {
            Ok((w, s)) => (w, s),
            Err(_) => { return; } 
        };
        
        if word_length == 0 { return; }

        if !word_collections_per_length.contains_key(&(word_length as u32)) {
            // TODO: backtrack here instead of panicking
            panic!("Cannot fill slot at all.");
        }

        let available_words = Rc::clone(&word_collections_per_length[&(word_length as u32)]);
        let slot = Slot::new(
                    *slot_id,
                    Rc::clone(&available_words), 
                    // Rc::clone(&(self.layout[(adjusted_vertical_idx - word_length_from_vertical as u32) as usize][(horizontal_idx - 1) as usize])), 
                    slot_cells
                );

        println!("Constructed slot {}", slot.get_slot_id());
        self.word_slots.push(slot);
        *slot_id += 1;
    }

    fn create_all_slots(&mut self, word_collections_per_length : &HashMap<u32, Rc<RefCell<BTreeSet<String>>>>) -> () 
    {
        let mut slot_id : u32 = 0;
        for vertical_idx in 0..self.height 
        {
            for horizontal_idx in 0..self.width 
            {
                let assigned_cell_states = self.set_cell_state(vertical_idx, horizontal_idx);
                if let Err(_) = self.restrict_neighborhood(vertical_idx, horizontal_idx, &assigned_cell_states) {
                    // backtrack
                }

                let at_end_of_height = vertical_idx == self.height - 1;
                let at_end_of_width = horizontal_idx == self.width - 1;
                let at_end_of_grid = at_end_of_height || at_end_of_width;
                let on_first_row = vertical_idx == 0;
                let on_first_col = horizontal_idx == 0;
                let encountered_clue_cell = assigned_cell_states.iter().any(|state| matches!(state, PossibleCellState::Clue(_)));

                if !on_first_row && !on_first_col && (encountered_clue_cell || at_end_of_grid) 
                {
                    if encountered_clue_cell || at_end_of_height 
                    {
                        self.create_slot(vertical_idx, horizontal_idx, &mut slot_id, &word_collections_per_length, encountered_clue_cell, "vertical");
                    }

                    if encountered_clue_cell || at_end_of_width 
                    {
                        self.create_slot(vertical_idx, horizontal_idx, &mut slot_id, &word_collections_per_length, encountered_clue_cell, "horizontal");
                    }
                }
            }
        }

        self.word_slots.sort_by(|slot1, slot2| slot1.get_suitable_word_set().len().cmp(&slot2.get_suitable_word_set().len()));
    }

    fn fill_slot(&mut self, slot_id: u32) -> Result<(), BTreeSet<(u32, u32)>> 
    {
        let slot_crossing_ids = self.get_slot(slot_id).unwrap().get_crossings();
        let mut already_attempted_words = BTreeSet::new();

        'selections: loop 
        {
            let nominated_word = 
            {
                let slot = self.get_slot(slot_id).unwrap();
                slot.nominate_word(&already_attempted_words).or(Err(slot_crossing_ids.clone()))?
            };

            println!("Crossing ids are: {:?}", slot_crossing_ids);
            for (idx, crossing_id) in &slot_crossing_ids 
            {
                println!("Iterating over ids");
                let crossing_slot = self.get_slot(*crossing_id).unwrap();

                if !crossing_slot.has_possibilities_remaining(slot_id, &nominated_word, *idx) 
                {
                    println!("Crossing slot {} did not have possibilities remaining.", crossing_id);
                    already_attempted_words.insert(nominated_word);
                    continue 'selections;
                }

                crossing_slot.determine_unsuitable_words_due_crossing_slot(slot_id, &nominated_word, *idx);
            }

            println!("In word allocation for slot id {}, chosen word is {}", slot_id, nominated_word);
        
            self.words_placed.insert(nominated_word.clone());
            self.get_slot(slot_id).unwrap().place_nominated_word(nominated_word);

            for (_, crossing_id) in slot_crossing_ids 
            {
                let crossing_slot = self.get_slot(crossing_id);
                crossing_slot.unwrap().apply_restrictions(slot_id);
            }
        
            return Ok(());
        };
    }

    fn fill_grid(&mut self) -> Result<(), LayoutError> 
    {
        // let mut already_tried_backtrackings_per_slot : HashMap<u32, BTreeSet<u32>> = HashMap::new();
        let mut slot_stack : Vec<u32> = Vec::new();
        for idx_to_add_to_stack in 0..self.word_slots.len() 
        {
            slot_stack.push(self.word_slots[idx_to_add_to_stack].get_slot_id()); 
        }

        while !slot_stack.is_empty() 
        {
            let curr_slot_id = slot_stack.pop().unwrap();
            println!("Trying slot id: {}", curr_slot_id);
            
            if let Err(crossings) = self.fill_slot(curr_slot_id) 
            {
                if crossings.is_empty() 
                {
                    println!("Found empty crossing_id set.");
                    return Err(LayoutError::NoPossibleDomainAfterRecursion);
                }

                println!("Pushing back slot id {}", curr_slot_id);
                slot_stack.push(curr_slot_id);

                // let already_tried = already_tried_backtrackings_per_slot.entry(curr_slot_id).or_default();

                let crossing_ids : BTreeSet<u32> = crossings.iter().map(|(_, id)| *id).collect();
                // let mut candidates : BTreeSet<&u32> = crossing_ids.difference(already_tried).collect();
                // if candidates.is_empty() 
                // {
                //     already_tried.clear();
                //     candidates = crossing_ids.iter().collect();
                // }

                let crossing_id = *crossing_ids.iter().filter(|id| { let slot = self.get_slot(**id); slot.unwrap().has_placed_word() }).last().unwrap();
                let mut crossing_slot = self.get_slot(crossing_id);
                crossing_slot.as_mut().unwrap().deallocate_and_discard_word();

                let secondary_crossing_ids = crossing_slot.unwrap().get_crossings();
                for (_, id) in secondary_crossing_ids 
                {
                    println!("Removing restrictions for secondary crossing id {}", id);
                    let secondary_crossing_slot = self.get_slot(id);
                    secondary_crossing_slot.unwrap().remove_unsuitable_words_related_to_slot_id(crossing_id);
                }

                // already_tried_backtrackings_per_slot.entry(curr_slot_id).or_default().insert(crossing_id);

                println!("Pushing slot id {} in stack", crossing_id);
                slot_stack.push(crossing_id);
            }
        }

        Ok(())
    }

    fn set_cell_state(&mut self, vertical_idx: u32, horizontal_idx: u32) -> BTreeSet<PossibleCellState> 
    {
        self.apply_first_row_column_restrictions(vertical_idx, horizontal_idx);
        let current_word_len : i32 = self.get_max_len(vertical_idx, horizontal_idx);

        let mut curr_cell = self.layout[vertical_idx as usize][horizontal_idx as usize].borrow_mut();
        let mut rng = rand::rng();
        let assigned_states : BTreeSet<PossibleCellState> = BTreeSet::new();

        let is_first_row = vertical_idx == 0;
        let is_first_col = horizontal_idx == 0;

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
                return Self::assign_clue_states(&mut *curr_cell, rng, assigned_states, is_first_row, is_first_col,false);
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
            return Self::assign_clue_states(&mut *curr_cell, rng, assigned_states, is_first_row, is_first_col,true);
        }
        else if curr_cell.can_still_be_clue() 
        {
            self.num_cells_accessed += 1;
            self.clues_placed += 1;
            return Self::assign_clue_states(&mut *curr_cell, rng, assigned_states, is_first_row, is_first_col,false);
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

    fn assign_letter(curr_cell: &mut GridCell, mut assigned_states: BTreeSet<PossibleCellState>) -> BTreeSet<PossibleCellState> 
    {
        curr_cell.assign_letter_state();
        assigned_states.insert(PossibleCellState::Letter);
        return assigned_states;
    }

    fn assign_clue_states(
        curr_cell: &mut GridCell, 
        mut rng: ThreadRng, 
        mut assigned_states: BTreeSet<PossibleCellState>, 
        is_first_row: bool, 
        is_first_col: bool, 
        randomize: bool) -> BTreeSet<PossibleCellState> 
    {
        let mut num_assigned_states = 0;
        let mut random_number: f32 = 0.0; // starts at 0.0 so at least one state is assigned

        while num_assigned_states < 2 && random_number < constants::THRESHOLD_PROBABILITY_FOR_SECOND_CLUE_STATE 
        {
            match curr_cell.assign_clue_state(&mut rng, &assigned_states, is_first_row, is_first_col) 
            {
                Some(state) => 
                {
                    assigned_states.insert(state);
                    num_assigned_states += 1;
                    random_number = if randomize { rng.random() } else { 0.0 };
                }
                None => break,
            }
        }

        assigned_states
    }

    fn apply_first_row_column_restrictions(&mut self, vertical_idx: u32, horizontal_idx: u32) -> () 
    {
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

    fn restrict_neighborhood(&mut self, vertical_idx: u32, horizontal_idx: u32, assigned_cell_states: &BTreeSet<PossibleCellState>) -> Result<(), LayoutError> 
    {
        if assigned_cell_states.iter().all(|state| matches!(state, PossibleCellState::Clue(_))) {

            if assigned_cell_states.iter().any(|state| matches!(state, PossibleCellState::Clue(SlotDirection::Right))) 
            {
                self.ensure_clear_slot(vertical_idx, horizontal_idx, SlotDirection::Right)?;
            }

            if assigned_cell_states.iter().any(|state| matches!(state, PossibleCellState::Clue(SlotDirection::RightOnBottomSide))) 
            {   
                self.ensure_clear_slot(vertical_idx, horizontal_idx, SlotDirection::RightOnBottomSide)?;
                self.ensure_correct_clue_cell_placement_near_boundary(vertical_idx, horizontal_idx, SlotDirection::RightOnBottomSide)?;
            }

            if assigned_cell_states.iter().any(|state| matches!(state, PossibleCellState::Clue(SlotDirection::Down))) 
            {
                self.ensure_clear_slot(vertical_idx, horizontal_idx, SlotDirection::Down)?;
            }

            if assigned_cell_states.iter().any(|state| matches!(state, PossibleCellState::Clue(SlotDirection::DownOnRightSide))) 
            {
                self.ensure_clear_slot(vertical_idx, horizontal_idx, SlotDirection::DownOnRightSide)?;
                self.ensure_correct_clue_cell_placement_near_boundary(vertical_idx, horizontal_idx, SlotDirection::DownOnRightSide)?;
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

    fn ensure_clear_slot(&mut self, vertical_idx: u32, horizontal_idx: u32, slot_direction: SlotDirection) -> Result<(), LayoutError> 
    {
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

    fn check_straight_clear_slot(&mut self, vertical_idx: u32, horizontal_idx: u32, dx: u32, dy: u32) -> Result<(), LayoutError> 
    {    
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

    fn ensure_correct_clue_cell_placement_near_boundary(&mut self, vertical_idx: u32, horizontal_idx: u32, slot_direction: SlotDirection) -> Result<(), LayoutError> 
    {
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

    fn is_start_of_angled_horizontal_slot(&self, vertical_idx: u32, horizontal_idx: u32) -> bool 
    {
        return vertical_idx > 0 && self.layout[(vertical_idx - 1) as usize][horizontal_idx as usize].borrow().is_clue_of_type(SlotDirection::RightOnBottomSide)
    }

    fn is_start_of_angled_vertical_slot(&self, vertical_idx: u32, horizontal_idx: u32) -> bool 
    {
        return horizontal_idx > 0 && self.layout[vertical_idx as usize][(horizontal_idx - 1) as usize].borrow().is_clue_of_type(SlotDirection::DownOnRightSide)
    }

    fn prevent_letter_underneath(&mut self, vertical_idx: u32, horizontal_idx: u32) -> () 
    {
        if vertical_idx == self.height - 1 {
            return;
        }

        self.layout[(vertical_idx + 1) as usize][horizontal_idx as usize].borrow_mut().force_clue();
    }

    fn prevent_letter_on_the_right(&mut self, vertical_idx: u32, horizontal_idx: u32) -> () 
    {
        if horizontal_idx == self.width - 1 {
            return;
        }

        self.layout[vertical_idx as usize][(horizontal_idx + 1) as usize].borrow_mut().force_clue();
    }
}
