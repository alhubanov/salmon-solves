use serde::{Serialize, Deserialize};
use rand::prelude::*;
use ahash::{AHashSet};

use std::rc::Rc;
use std::cell::RefCell;

use crate::grid_scandi::gridcell::CellType;
use crate::grid_scandi::slot::CellType::Letter;
use crate::grid_scandi::slot::CellType::Clue;
use crate::grid_scandi::clue;
use crate::grid_scandi::letter;

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
    slot_cells: Vec<Rc<RefCell<GridCell>>>,
    associated_clue_cell: Rc<RefCell<GridCell>>,
    selected_word: Option<String>,
    dictionary: Rc<RefCell<Dictionary>>,
    available_discarded_candidates: Vec<String>
}

impl Slot {
    pub fn new(
        slot_id: u32,
        slot_cells: Vec<Rc<RefCell<GridCell>>>,
        associated_clue_cell: Rc<RefCell<GridCell>>,
        dictionary: Rc<RefCell<Dictionary>>
    ) 
    -> Self 
    {

        let selected_word = None;
        let available_discarded_candidates = Vec::new();

        Self { 
            slot_id, 
            slot_cells,
            associated_clue_cell,
            selected_word, 
            dictionary, 
            available_discarded_candidates
        }
    }

    pub fn get_slot_id(&self) -> u32 
    {
        self.slot_id
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

    pub fn get_crossings(&self) -> AHashSet<(u32, u32)> 
    {

        let mut crossing_slot_ids : AHashSet<(u32, u32)> = AHashSet::new();
        for (idx, cell) in self.slot_cells.iter().enumerate() 
        {
            let cell_foreign_crossing_id : Option<u32> = cell.borrow().slot_ids.as_ref().unwrap().iter().find(|elem| *elem != &self.slot_id).copied();
            if cell_foreign_crossing_id.is_some() 
            {
                crossing_slot_ids.insert((idx as u32, cell_foreign_crossing_id.unwrap()));
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
                Some(Letter(letter)) => pattern[idx] = Some(letter.get_cell_value()),
                Some(Clue(_)) => panic!("Encountered clue cell in slot cells."),
                None => pattern[idx] = None,
            }
        }

        return pattern;
    }

    pub fn get_current_candidate_count(&self) -> usize
    {
        let word_len = self.slot_cells.len();
        let pattern = self.get_current_pattern();

        let borrowed_dictionary = self.dictionary.borrow_mut();
        let suitable_candidates = borrowed_dictionary.get_candidates(word_len, &pattern);

        suitable_candidates.len()
    }

    pub fn nominate_word(&mut self, already_attempted_words: &AHashSet<String>, rng: &mut dyn Rng) -> Result<String, LayoutError> 
    {   
        let word_len = self.slot_cells.len();
        let pattern = self.get_current_pattern();

        let borrowed_dictionary = self.dictionary.borrow_mut();
        let suitable_candidates = borrowed_dictionary.get_candidates(word_len, &pattern);

        let mut filtered_suitable_candidates: Vec<&SlotCandidate> = suitable_candidates
            .into_iter()
            .filter(|candidate| 
                        candidate.get_assigned_slot_id() == None &&
                        // candidate.get_conflicts().iter().map(|conflict| conflict.get_restricted_slot_id()).all(|restricted_id| restricted_id != self.slot_id) &&  
                        !self.available_discarded_candidates.contains(candidate.get_word()) && 
                        !already_attempted_words.contains(candidate.get_word()))
            .collect();

        if filtered_suitable_candidates.is_empty() { return Err(LayoutError::NoPossibleDomain); }
        
        filtered_suitable_candidates.sort();
        let idx = rng.random_range(0..filtered_suitable_candidates.len());
        let sampled_word = filtered_suitable_candidates[idx].get_word().clone();

        return Ok(sampled_word);
    }

    fn get_prospective_pattern(&self, slot_id: u32, nominated_word: &String, idx_of_crossing_letter: u32) -> Vec<Option<char>> 
    {
        let mut pattern = Vec::new();
        for cell in self.slot_cells.iter()
        {
            if cell.borrow().slot_ids.as_ref().is_some_and(|ids| ids.contains(&slot_id)) 
            {
                pattern.push(Some(nominated_word.as_bytes()[idx_of_crossing_letter as usize] as char));
            }
            else 
            {
                let borrowed_cell = cell.borrow();
                match &borrowed_cell.cell 
                {
                    Some(Letter(letter)) => pattern.push(Some(letter.get_cell_value())),
                    Some(Clue(_)) => panic!("Encountered clue cell in slot cells."),
                    None => pattern.push(None),
                }
            }
        }

        return pattern;
    }

    pub fn has_possibilities_remaining(&self, slot_id: u32, nominated_word: &String, idx_of_crossing_letter: u32) -> bool 
    {
        if self.selected_word.is_some() {
            return true;
        }

        let pattern = self.get_prospective_pattern(slot_id, nominated_word, idx_of_crossing_letter);
        self.dictionary.borrow().get_candidates(self.slot_cells.len(), &pattern).len() > 0
    }

    pub fn place_nominated_word_and_associated_clue(&mut self, nominated_word: String) -> () 
    {   
        let mut dictionary = self.dictionary.borrow_mut();

        // Assume a valid candidate always exists     
        if let Some(candidate) = dictionary
                                     .get_words_for_length_mut(self.slot_cells.len())
                                     .iter_mut()
                                     .find(|candidate| candidate.get_word() == &nominated_word)
        {
            candidate.set_assigned_slot_id(self.slot_id);
        }
        
        // There is a possibility to use .iter.zip() here to avoid calling .nth(). 
        // It is only faster though if the majority of instances, where this loop runs, consist mostly of None cells. 
        for (idx, cell) in self.slot_cells.iter().enumerate() {
            if cell.borrow().cell.is_none() {
                cell.borrow_mut().cell = Some(CellType::Letter(letter::Letter::new(nominated_word.chars().nth(idx).unwrap())));
            }
        }

        // TODO: add proper hints to clues
        self.associated_clue_cell.borrow_mut().cell = Some(CellType::Clue(clue::Clue::new(&"".to_string())));

        self.selected_word = Some(nominated_word);
    }

    pub fn deallocate_and_discard_word<F>(&mut self, crossing_slot_has_word: F) -> ()
    where
        F: Fn(u32) -> bool,
    {
        for cell in self.slot_cells.iter() 
        {
            let borrowed = cell.borrow();
            let owned_by_active_crossing = borrowed.slot_ids
                .as_ref()
                .map(|ids| ids.iter().any(|&id| id != self.slot_id && crossing_slot_has_word(id)))
                .unwrap_or(false);

            drop(borrowed);

            if !owned_by_active_crossing 
            {
                cell.borrow_mut().cell = None;
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