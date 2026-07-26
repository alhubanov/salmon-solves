use std::cmp::max;
use std::collections::VecDeque;
use rand::{prelude::*};
use rand::rand_core::Rng;
use wasm_bindgen::prelude::*;
use serde::Serialize;

use ahash::{AHashMap, AHashSet};

use std::rc::Rc;
use std::cell::RefCell;

#[cfg(test)]
mod unit_tests;
#[cfg(test)]
mod test_helpers;

mod clue;
mod letter;
pub mod gridcell;
mod slot;
mod slot_candidate;
mod dictionary;
pub mod run_stats;

use slot::Slot;
use slot::SlotDirection;
use gridcell::{GridCell, PossibleCellState};
use dictionary::Dictionary;
use crate::grid::LayoutError;
use crate::grid::Grid;
use crate::grid_scandi::run_stats::RunStats;

mod constants {
    pub const TARGET_CLUE_DENSITY : f32                        = 0.25;
    pub const THRESHOLD_PROBABILITY_FOR_SECOND_CLUE_STATE: f32 = 0.6;
    pub const _MAXIMUM_WORD_LENGTH : i32                       = 10; // unused currently
    pub const MEDIAN_WORD_LENGTH : i32                         = 6; 
    pub const PER_LETTER_LENGTH_PRESSURE : f32                 = 0.08;
    pub const _DEFICIT_RATE_DEFAULT : f32                      = 0.35; // unused currently
}

static WORDS: &str = include_str!("../word_files/common_english_words.txt");

pub struct ScandiGrid {
    width: u32,
    height: u32,
    layout: Vec<Vec<Rc<RefCell<GridCell>>>>,
    clues_placed: u32,
    num_cells_accessed: u32,
    word_slots: Vec<Slot>,
    slot_id_pos_map: AHashMap<u32, u32>
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
        let word_slots = Vec::new();
        let slot_id_pos_map = AHashMap::new();

        Self { width, height, layout, clues_placed, num_cells_accessed, word_slots, slot_id_pos_map }
    }

    fn construct(&mut self, rng: &mut dyn Rng, max_depth: u32, run_stats: &mut RunStats) -> Result<(), LayoutError> 
    {
        let dictionary = Rc::new(RefCell::new(Dictionary::build(WORDS)));

        self.create_all_slots(&dictionary, rng);
        self.fill_grid(rng, max_depth, run_stats)?;

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

    fn layout(&self) -> Result<JsValue, JsValue> {
        let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true).serialize_missing_as_null(true);
        self.layout
            .serialize(&serializer)
            .map_err(|e| e.to_string().into())
    }
}

impl ScandiGrid {

    fn get_slot_mut(&mut self, slot_id: u32) -> &mut Slot 
    {
        &mut self.word_slots[self.slot_id_pos_map[&slot_id] as usize]
    }

    fn get_len_and_associated_clue_coords(&self, vertical_idx: u32, horizontal_idx: u32, end_of_grid: bool, orientation: &str) -> Result<(i32, u32, u32), LayoutError> 
    {
        let mut slot_count = if end_of_grid { 0 } else { -1 };
        let mut curr_vertical_idx = vertical_idx;
        let mut curr_horizontal_idx = horizontal_idx;

        let clue_direction = if orientation == "vertical" { SlotDirection::Down } else { SlotDirection::Right };

        let (word_len, associated_clue_vertical_idx, associated_clue_horizontal_idx) = loop 
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
                break (slot_count, curr_vertical_idx, curr_horizontal_idx);
            }
            else if active_orientation_idx == 0 
            {
                slot_count += 1;

                if orientation == "vertical" 
                {
                    let out_horizontal_idx = if curr_horizontal_idx == 0 { curr_horizontal_idx } else { curr_horizontal_idx - 1 }; 
                    break (slot_count, curr_vertical_idx, out_horizontal_idx); 
                } 
                else 
                { 
                    let out_vertical_idx = if curr_vertical_idx == 0 { curr_vertical_idx } else { curr_vertical_idx - 1 }; 
                    break (slot_count, out_vertical_idx, curr_horizontal_idx); 
                }
            }

            slot_count += 1;
            if orientation == "vertical" { curr_vertical_idx -= 1; } else { curr_horizontal_idx -= 1; }
        };

        return Ok((word_len, associated_clue_vertical_idx, associated_clue_horizontal_idx));
    }

    fn get_max_len(&self, vertical_idx: u32, horizontal_idx: u32) -> i32 
    {
        let (vertical_word_len, _, _) = self.get_len_and_associated_clue_coords(vertical_idx, horizontal_idx, false, "vertical").unwrap_or((0, 0, 0));
        let (horizontal_word_len, _, _) = self.get_len_and_associated_clue_coords(vertical_idx, horizontal_idx, false, "horizontal").unwrap_or((0, 0, 0));

        return max(vertical_word_len, horizontal_word_len);
    }

    fn get_slot_attributes(&self, mut vertical_idx: u32, mut horizontal_idx: u32, end_of_grid: bool, slot_id: Option<u32>, orientation: &str) -> Result<(i32, Vec<Rc<RefCell<GridCell>>>, Rc<RefCell<GridCell>>), LayoutError> 
    {
        let start_position = if end_of_grid { 0 } else { -1 };

        let mut slot_cells : Vec<Rc<RefCell<GridCell>>> = Vec::new();
        let (word_len, associated_clue_vertical_idx, associated_clue_horizontal_idx) = self.get_len_and_associated_clue_coords(vertical_idx, horizontal_idx, end_of_grid, orientation)?;
        let associated_clue_cell = Rc::clone(&self.layout[associated_clue_vertical_idx as usize][associated_clue_horizontal_idx as usize]);

        for idx in start_position..word_len 
        {
            if idx == -1 
            {
                if orientation == "vertical" { vertical_idx -= 1; } else { horizontal_idx -= 1; } 
                continue; 
            }

            self.layout[vertical_idx as usize][horizontal_idx as usize].borrow_mut().add_slot_id(slot_id);
            slot_cells.push(Rc::clone(&self.layout[vertical_idx as usize][horizontal_idx as usize]));  

            if idx < word_len - 1 {
                if orientation == "vertical" { vertical_idx -= 1; } else { horizontal_idx -= 1; } 
            }
        }

        slot_cells.reverse();
        return Ok((word_len, slot_cells, associated_clue_cell));
    }

    fn create_slot(&mut self, 
                    vertical_idx: u32, 
                    horizontal_idx: u32, 
                    slot_id: &mut u32, 
                    dictionary: &Rc<RefCell<Dictionary>>,
                    encountered_clue_cell: bool,
                    orientation: &str)  
    {
        let consider_last_cell = !encountered_clue_cell;
        let slot_attributes = self.get_slot_attributes(vertical_idx, horizontal_idx, consider_last_cell, Some(*slot_id), orientation);

        let (word_length, slot_cells, associated_clue_cell) = match slot_attributes 
        {
            Ok((w, s, c)) => (w, s, c),
            Err(_) => { return; } 
        };
        
        if word_length == 0 { return; }

        if !dictionary.borrow().get_words().contains_key(&(word_length as usize)) {
            // TODO: backtrack here instead of panicking
            panic!("Cannot fill slot at all.");
        }

        let available_candidates = Rc::clone(dictionary);
        let slot = Slot::new(
                    *slot_id,
                    slot_cells,
                    associated_clue_cell,
                    available_candidates
                );

        self.word_slots.push(slot);
        *slot_id += 1;
    }

    fn create_all_slots(&mut self, dictionary: &Rc<RefCell<Dictionary>>, rng: &mut dyn Rng) -> () 
    {
        let mut slot_id : u32 = 0;
        for vertical_idx in 0..self.height 
        {
            for horizontal_idx in 0..self.width 
            {
                let assigned_cell_states = self.set_cell_state(vertical_idx, horizontal_idx, rng);
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
                        self.create_slot(vertical_idx, horizontal_idx, &mut slot_id, dictionary, encountered_clue_cell, "vertical");
                    }

                    if encountered_clue_cell || at_end_of_width 
                    {
                        self.create_slot(vertical_idx, horizontal_idx, &mut slot_id, dictionary, encountered_clue_cell, "horizontal");
                    }
                }
            }
        }

        // self.word_slots.sort_by(|slot1, slot2| (*slot1.get_suitable_word_set()).len().cmp(&(*slot2.get_suitable_word_set()).len()));
        self.word_slots.sort_by_key(|slot| slot.get_crossings().len());

        for (idx, slot) in self.word_slots.iter().enumerate()
        {
            self.slot_id_pos_map.insert(slot.get_slot_id(), idx as u32);
        }
    }

    fn propagate_changes_across_crossing_slots(&mut self, slot_id: u32, domain: AHashSet<String>, max_depth: u32) -> bool 
    {
        let domain = Rc::new(domain);
        let mut visited: AHashSet<u32> = AHashSet::new();
        visited.insert(slot_id);

        let mut propagation_queue : VecDeque<((u32, u32), u32, Rc<AHashSet<String>>, u32)> = self.get_slot_mut(slot_id)
                                                                                                 .get_crossings()
                                                                                                 .into_iter()
                                                                                                 .map(|(a, b)| ((a, b), slot_id, Rc::clone(&domain), 1))
                                                                                                 .collect();
        while !propagation_queue.is_empty() 
        {
            let ((idx, slot_id_to_propagate_to), origin_id, restricting_domain, depth) = propagation_queue.pop_front().unwrap();

            if depth > max_depth || !visited.insert(slot_id_to_propagate_to) 
            {
                continue;
            }

            let curr_slot = self.get_slot_mut(slot_id_to_propagate_to);
            let remaining_word_candidates = Rc::new(curr_slot.get_remaining_word_candidates(origin_id, &restricting_domain, idx));

            if remaining_word_candidates.len() == 0
            {
                return false;
            }

            if curr_slot.get_current_candidate_count() == remaining_word_candidates.len()
            {
                continue;
            }

            let tuples_to_propagate : VecDeque<((u32, u32), u32, Rc<AHashSet<String>>, u32)> = curr_slot
                .get_crossings()
                .into_iter()
                .filter(|(_, crossing_id)| *crossing_id != origin_id)
                .map(|(a, b)| ((a, b), slot_id_to_propagate_to, Rc::clone(&remaining_word_candidates), depth + 1))
                .collect();
            propagation_queue.extend(tuples_to_propagate);
        }

        true
    }

    fn fill_slot(&mut self, slot_id: u32, rng: &mut dyn Rng, placement_order: &mut usize, max_depth: u32) -> Result<(), Vec<(u32, u32)>> 
    {
        let mut already_attempted_words = AHashSet::new();
        let slot_crossing_ids = self.get_slot_mut(slot_id).get_crossings();

        'selections: loop 
        {
            let nominated_word = match self.get_slot_mut(slot_id).nominate_word(&already_attempted_words, rng) 
            {
                Ok(word) => word,
                Err(_) => return Err(slot_crossing_ids)
            };

            let mut domain = AHashSet::new();
            domain.insert(nominated_word.clone());
            if !self.propagate_changes_across_crossing_slots(slot_id, domain, max_depth)
            {
                already_attempted_words.insert(nominated_word.clone());
                continue 'selections;
            }

            self.get_slot_mut(slot_id).place_nominated_word_and_associated_clue(nominated_word.clone(), placement_order.clone());
            self.invalidate_slot_and_crossings(slot_id);
            *placement_order += 1;

            return Ok(());
        };
    }

    fn fill_grid(&mut self, rng: &mut dyn Rng, max_depth: u32, run_stats: &mut RunStats) -> Result<(), LayoutError> 
    {
        let mut placement_order : usize = 0;
        const MAX_BACKTRACKS_BEFORE_RESTART: u32 = 100;
        let mut backtracking_count = 0;

        while self.word_slots.iter().filter(|slot| !slot.has_placed_word()).peekable().peek().is_some()
        {
            let unplaced_ids: Vec<u32> = self.word_slots.iter()
                                                        .filter(|slot| !slot.has_placed_word())
                                                        .map(|slot| slot.get_slot_id())
                                                        .collect();

            // pick the slot with feweste candidates left
            let curr_slot_id = unplaced_ids.into_iter()
                                            .map(|id| {
                                                let count = self.get_slot_mut(id).get_current_candidate_count();
                                                (id, count)
                                            })
                                            .min_by_key(|&(_, count)| count)
                                            .map(|(id, _)| id)
                                            .unwrap();

            if let Err(crossings) = self.fill_slot(curr_slot_id, rng, &mut placement_order, max_depth) 
            {
                backtracking_count += 1;
                run_stats.increment_total_backtracks();

                if backtracking_count > MAX_BACKTRACKS_BEFORE_RESTART 
                {
                    // println!("Backtracking count: {}", backtracking_count);
                    // println!("Exceeded backtracking threshold...");

                    run_stats.record_backtracking_count_for_attempt(backtracking_count);
                    return Err(LayoutError::ExceededBacktrackingThreshold);
                }

                if crossings.is_empty() 
                {
                    return Err(LayoutError::NoPossibleDomainAfterRecursion);
                }

                let candidates: Vec<u32> = crossings.iter()
                                                    .map(|(_, id)| *id)
                                                    .filter(|id| self.get_slot_mut(*id).has_placed_word())
                                                    .collect();

                let crossing_id = candidates.into_iter()
                                            .max_by_key(|id| self.get_slot_mut(*id).get_placement_order())
                                            .unwrap();

                // Snapshot which slots currently hold a placed word BEFORE deallocating,
                // so deallocation can tell which shared cells are still owned by an active crossing.
                let placed_word_slot_ids: AHashSet<u32> = self.word_slots.iter()
                                                                         .filter(|s| s.has_placed_word() && s.get_slot_id() != crossing_id)
                                                                         .map(|s| s.get_slot_id())
                                                                         .collect();

                let crossing_slot = self.get_slot_mut(crossing_id);
                crossing_slot.deallocate_and_discard_word(
                    |id| placed_word_slot_ids.contains(&id)
                );
                self.invalidate_slot_and_crossings(crossing_id);
            }
        }

        // println!("Backtracking count: {}", backtracking_count);
        run_stats.record_backtracking_count_for_attempt(backtracking_count);

        Ok(())
    }

    fn invalidate_slot_and_crossings(&mut self, slot_id: u32) 
    {
        let crossings = self.get_slot_mut(slot_id).get_crossings();
        self.get_slot_mut(slot_id).invalidate_candidate_count_cache();

        for (_, crossing_id) in crossings
        {
            self.get_slot_mut(crossing_id).invalidate_candidate_count_cache();
        }
    }

    fn set_cell_state(&mut self, vertical_idx: u32, horizontal_idx: u32, rng: &mut dyn Rng) -> Vec<PossibleCellState> 
    {
        self.apply_first_row_column_restrictions(vertical_idx, horizontal_idx);
        let current_word_len : i32 = self.get_max_len(vertical_idx, horizontal_idx);

        let mut curr_cell = self.layout[vertical_idx as usize][horizontal_idx as usize].borrow_mut();
        let assigned_states : Vec<PossibleCellState> = Vec::new();

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

    fn assign_letter(curr_cell: &mut GridCell, mut assigned_states: Vec<PossibleCellState>) -> Vec<PossibleCellState> 
    {
        curr_cell.assign_letter_state();
        assigned_states.push(PossibleCellState::Letter);
        return assigned_states;
    }

    fn assign_clue_states(
        curr_cell: &mut GridCell, 
        rng: &mut dyn Rng, 
        mut assigned_states: Vec<PossibleCellState>, 
        is_first_row: bool, 
        is_first_col: bool, 
        randomize: bool) -> Vec<PossibleCellState> 
    {
        let mut num_assigned_states = 0;
        let mut random_number: f32 = 0.0; // starts at 0.0 so at least one state is assigned

        while num_assigned_states < 2 && random_number < constants::THRESHOLD_PROBABILITY_FOR_SECOND_CLUE_STATE 
        {
            match curr_cell.assign_clue_state(rng, &assigned_states, is_first_row, is_first_col) 
            {
                Some(state) => 
                {
                    assigned_states.push(state);
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

    fn restrict_neighborhood(&mut self, vertical_idx: u32, horizontal_idx: u32, assigned_cell_states: &Vec<PossibleCellState>) -> Result<(), LayoutError> 
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
