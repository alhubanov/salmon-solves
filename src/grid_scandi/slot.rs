use serde::{Serialize, Deserialize};
use rand::prelude::*;
use ahash::{AHashSet};

use std::rc::Rc;
use std::cell::RefCell;

use crate::grid_scandi::gridcell::CellType;
use crate::grid_scandi::slot::CellType::Letter;
use crate::grid_scandi::slot::CellType::Clue;

use crate::grid_scandi::LayoutError;
use crate::grid_scandi::gridcell::GridCell;
use crate::grid_scandi::slot_candidate::SlotCandidate;
use crate::grid_scandi::dictionary::Dictionary;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, Copy, Clone, Hash)]
pub enum SlotDirection {
    Down,
    DownOnRightSide,
    Right,
    RightOnBottomSide
}

#[derive(PartialEq, Eq, Debug)]
pub struct Slot {
    slot_id: u32,
    slot_direction: SlotDirection,
    slot_cells: Vec<Rc<RefCell<GridCell>>>,
    associated_clue_cell: Rc<RefCell<GridCell>>,
    selected_word: Option<String>,
    dictionary: Rc<RefCell<Dictionary>>,
    available_discarded_candidates: Vec<String>,
    placement_order: Option<usize>,
    cached_candidate_count: Option<usize>
}

impl Slot {
    pub fn new(
        slot_id: u32,
        slot_direction: SlotDirection,
        slot_cells: Vec<Rc<RefCell<GridCell>>>,
        associated_clue_cell: Rc<RefCell<GridCell>>,
        dictionary: Rc<RefCell<Dictionary>>
    ) 
    -> Self 
    {

        let selected_word = None;
        let available_discarded_candidates = Vec::new();
        let placement_order = None;
        let cached_candidate_count = None;

        Self 
        { 
            slot_id, 
            slot_direction,
            slot_cells,
            associated_clue_cell,
            selected_word, 
            dictionary, 
            available_discarded_candidates,
            placement_order,
            cached_candidate_count
        }
    }

    pub fn get_slot_id(&self) -> u32 
    {
        self.slot_id
    }

    pub fn get_placement_order(&self) -> i32
    {
        if self.placement_order == None
        {
            return -1;
        }

        self.placement_order.unwrap() as i32
    }

    #[cfg(test)]
    pub fn get_selected_word(&self) -> &Option<String>
    {
        &self.selected_word
    }

    #[cfg(test)]
    pub fn get_cells(&self) -> &Vec<Rc<RefCell<GridCell>>>
    {
        &self.slot_cells
    }

    pub fn get_crossings(&self) -> Vec<(u32, u32)> 
    {

        let mut crossing_slot_ids : Vec<(u32, u32)> = Vec::new();
        for (idx, gridcell) in self.slot_cells.iter().enumerate() 
        {
            let borrowed_gridcell = gridcell.borrow();
            let cell_foreign_crossing_id : Option<&u32> = borrowed_gridcell.get_foreign_slot_id_from_letter_cell(self.slot_id);
            if cell_foreign_crossing_id.is_some() 
            {
                crossing_slot_ids.push((idx as u32, *cell_foreign_crossing_id.unwrap()));
            }
        }

        crossing_slot_ids
    }

    pub fn has_placed_word(&self) -> bool 
    {
        self.selected_word.is_some()
    }

    pub fn get_current_pattern(&self) -> Vec<Option<char>>
    {
        let word_len = self.slot_cells.len();
        let mut pattern = vec![None; word_len]; 

        for (idx, cell) in self.slot_cells.iter().enumerate()
        {
            let borrowed_cell = cell.borrow();
            match &borrowed_cell.cell 
            {
                Some(Letter(letter, _)) => pattern[idx] = *letter,
                Some(Clue(_)) => panic!("Encountered clue cell in slot cells."),
                None => pattern[idx] = None,
            }
        }

        return pattern;
    }

    // pub fn get_current_candidate_count(&self) -> usize
    // {
    //     let word_len = self.slot_cells.len();
    //     let pattern = self.get_current_pattern();

    //     let borrowed_dictionary = self.dictionary.borrow_mut();
    //     let suitable_candidates = borrowed_dictionary.get_candidates(word_len, &pattern);

    //     suitable_candidates.len()
    // }

    pub fn get_current_candidate_count(&mut self) -> usize 
    {
        if let Some(count) = self.cached_candidate_count 
        {
            return count;
        }

        let word_len = self.slot_cells.len();
        let pattern = self.get_current_pattern();
        let count = self.dictionary.borrow().get_candidate_count(word_len, &pattern);

        self.cached_candidate_count = Some(count);
        count
    }

    pub fn invalidate_candidate_count_cache(&mut self) 
    {
        self.cached_candidate_count = None;
    }

    fn get_prospective_pattern(&self, slot_id: u32, letter: char) -> Vec<Option<char>> 
    {
        let mut pattern = Vec::new();
        for gridcell in self.slot_cells.iter()
        {
            if gridcell.borrow().contains_slot_id(slot_id)  
            {
                pattern.push(Some(letter));
            }
            else 
            {
                let borrowed_cell = gridcell.borrow();
                match &borrowed_cell.cell 
                {
                    Some(Letter(letter, _)) => pattern.push(*letter),
                    Some(Clue(_)) => panic!("Encountered clue cell in slot cells."),
                    None => pattern.push(None),
                }
            }
        }

        pattern
    }

    pub fn get_remaining_word_candidates(&self, slot_id: u32, domain: &AHashSet<String>, idx_of_crossing_letter: u32) -> AHashSet<String>
    {
        let distinct_letters: AHashSet<char> = domain.iter()
                                                     .map(|word| word.as_bytes()[idx_of_crossing_letter as usize] as char)
                                                     .collect();

        let mut candidates : AHashSet<String> = AHashSet::new();
        for letter in distinct_letters
        {
            let pattern = self.get_prospective_pattern(slot_id, letter);
            let candidates_for_letter : AHashSet<String> = self.dictionary.borrow()
                                                                          .get_candidates(self.slot_cells.len(), &pattern)
                                                                          .iter()
                                                                          .map(|slot_candidate| slot_candidate.get_word().clone())
                                                                          .collect();
            candidates.extend(candidates_for_letter);
        }

        candidates
    }

    pub fn nominate_word_and_clue(&mut self, already_attempted_words: &AHashSet<String>, rng: &mut dyn Rng) -> Result<(String, String), LayoutError> 
    {   
        let word_len = self.slot_cells.len();
        let pattern = self.get_current_pattern();

        let borrowed_dictionary = self.dictionary.borrow_mut();
        let suitable_candidates = borrowed_dictionary.get_candidates(word_len, &pattern);

        let filtered_suitable_candidates: Vec<&SlotCandidate> = suitable_candidates
            .into_iter()
            .filter(|candidate| 
                        candidate.get_assigned_slot_id() == None &&
                        // candidate.get_conflicts().iter().map(|conflict| conflict.get_restricted_slot_id()).all(|restricted_id| restricted_id != self.slot_id) &&  
                        !self.available_discarded_candidates.contains(candidate.get_word()) && 
                        !already_attempted_words.contains(candidate.get_word()))
            .collect();

        if filtered_suitable_candidates.is_empty() { return Err(LayoutError::NoPossibleDomain); }
        
        // filtered_suitable_candidates.sort();
        let idx = rng.random_range(0..filtered_suitable_candidates.len());
        let sampled_word = filtered_suitable_candidates[idx].get_word().clone();
        let sampled_clue = filtered_suitable_candidates[idx].get_clue().clone();

        return Ok((sampled_word, sampled_clue));
    }

    fn assign_associated_gridcell(&mut self, clue: String) -> ()
    {
        let mut clue_vec = Vec::new();
        clue_vec.push((clue, self.slot_id, self.slot_direction));

        self.associated_clue_cell.borrow_mut().cell = Some(CellType::Clue(clue_vec));
    }

    pub fn place_nominated_word_and_associated_clue(&mut self, nominated_word_and_clue: (String, String), placement_order: usize) -> () 
    {   
        {
            let mut dictionary = self.dictionary.borrow_mut();

            // Assume a valid candidate always exists     
            if let Some(candidate) = dictionary
                                        .get_words_for_length_mut(self.slot_cells.len())
                                        .iter_mut()
                                        .find(|candidate| candidate.get_word() == &nominated_word_and_clue.0)
            {
                candidate.set_assigned_slot_id(self.slot_id);
            }
        }
        
        // There is a possibility to use .iter.zip() here to avoid calling .nth(). 
        // It is only faster though if the majority of instances, where this loop runs, consist mostly of None cells. 
        for (idx, gridcell) in self.slot_cells.iter().enumerate() 
        {
            if gridcell.borrow().has_no_letter_assigned() 
            {
                gridcell.borrow_mut().assign_letter(nominated_word_and_clue.0.chars().nth(idx).unwrap());
            }
        }

        let clue = nominated_word_and_clue.1;
        if self.associated_clue_cell.borrow_mut().cell.is_none()
        {
            self.assign_associated_gridcell(clue);
        }
        else // this assumes that associated clue cell cannot possibly be a letter
        {
            self.associated_clue_cell.borrow_mut().append_clue(self.slot_id, self.slot_direction, clue);
        }

        self.selected_word = Some(nominated_word_and_clue.0);
        self.placement_order = Some(placement_order);
    }

    pub fn deallocate_and_discard_word<F>(&mut self, mut crossing_slot_has_word: F) -> ()
    where
        F: Fn(u32) -> bool,
    {
        for cell in self.slot_cells.iter() 
        {
            let borrowed = cell.borrow();
            let owned_by_active_crossing = borrowed.get_slot_ids_owned_by_crossing_slot(self.slot_id, &mut crossing_slot_has_word);

            drop(borrowed);

            if !owned_by_active_crossing 
            {
                cell.borrow_mut().discard_letter();
            }
        }

        // It is not clear to me whether discarding words permanently is good enough here.
        if self.selected_word != None 
        {
            self.available_discarded_candidates.push(self.selected_word.clone().unwrap());
            let mut dictionary = self.dictionary.borrow_mut();

            // Assume a valid candidate always exists   
            if let Some(candidate) = dictionary
                                     .get_words_for_length_mut(self.slot_cells.len())
                                     .iter_mut()
                                     .find(|candidate| candidate.get_word() == &self.selected_word.clone().unwrap())
            {
                candidate.clear_assigned_slot_id();
            }
        }
        
        self.selected_word = None;
    } 
}