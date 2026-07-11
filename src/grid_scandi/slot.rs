use serde::{Serialize, Deserialize};
use rand::prelude::*;
use ahash::{AHashSet};

use std::rc::Rc;
use std::cell::RefCell;
use std::cell::Ref;

use crate::grid_scandi::LayoutError;
use crate::grid_scandi::clue;
use crate::grid_scandi::gridcell::GridCell;

use crate::grid_scandi::gridcell::CellType;
use crate::grid_scandi::letter;
use crate::grid_scandi::slot_candidate::SlotCandidate;

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
    available_candidates: Rc<RefCell<Vec<SlotCandidate>>>,
    available_discarded_candidates: Vec<String>
}

impl Slot {
    pub fn new(
        slot_id: u32,
        slot_cells: Vec<Rc<RefCell<GridCell>>>,
        associated_clue_cell: Rc<RefCell<GridCell>>,
        available_candidates: Rc<RefCell<Vec<SlotCandidate>>>
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
            available_candidates, 
            available_discarded_candidates
        }
    }

    pub fn get_suitable_word_set(&self) -> Ref<'_, Vec<SlotCandidate>> 
    {
        self.available_candidates.borrow()
    }

    pub fn get_slot_id(&self) -> u32 
    {
        self.slot_id
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

    pub fn nominate_word(&mut self, already_attempted_words: &AHashSet<String>, rng: &mut dyn Rng) -> Result<String, LayoutError> 
    {    
        let borrowed_available_candidates = self.available_candidates.borrow();
        let mut suitable_candidates: Vec<&SlotCandidate> = borrowed_available_candidates
                                                .iter()
                                                .filter(|candidate| 
                                                            candidate.get_assigned_slot_id() == None &&
                                                            candidate.get_conflicts().iter().map(|conflict| conflict.get_restricted_slot_id()).all(|restricted_id| restricted_id != self.slot_id) &&  
                                                            !self.available_discarded_candidates.contains(candidate.get_word()) && 
                                                            !already_attempted_words.contains(candidate.get_word()))
                                                .collect();

        if suitable_candidates.is_empty() { return Err(LayoutError::NoPossibleDomain); }
        
        suitable_candidates.sort();
        let idx = rng.random_range(0..suitable_candidates.len());
        let sampled_word = suitable_candidates[idx].get_word().clone();

        return Ok(sampled_word);
    }

    fn determine_crossing_points(&self, slot_id: u32, nominated_word: &String, idx_of_crossing_letter: u32) -> Vec<(usize, u8)> 
    {
        self.slot_cells
            .iter()
            .enumerate()
            .filter_map(|(idx, cell)| {
                if cell.borrow().slot_ids.as_ref().is_some_and(|ids| ids.contains(&slot_id)) 
                {
                    Some((idx, nominated_word.as_bytes()[idx_of_crossing_letter as usize]))
                } 
                else 
                {
                    None
                }
            })
            .collect()
    }

    pub fn remove_slot_candidate_restrictions(&mut self, slot_id: u32) 
    {
        for candidate in self.available_candidates.borrow_mut().iter_mut() 
        {
            candidate.remove_restrictions(self.slot_id, slot_id);
        }
    }

    pub fn has_possibilities_remaining(&self, slot_id: u32, nominated_word: &String, idx_of_crossing_letter: u32) -> bool 
    {
        if self.selected_word.is_some() {
            return true;
        }

        let required_letters = self.determine_crossing_points(slot_id, nominated_word, idx_of_crossing_letter);

        let borrowed_available_candidates = self.available_candidates.borrow(); 
        let num_suitable_words = borrowed_available_candidates.iter()
                                                .filter(
                                                    |candidate| 
                                                    {
                                                        !candidate.get_conflicts().iter().map(|conflict| conflict.get_restricted_slot_id()).any(|restricted_id| restricted_id == slot_id) &&
                                                        required_letters.iter().all(|&(idx, letter)| candidate.get_word().as_bytes()[idx] == letter)
                                                    })
                                                .count();

        num_suitable_words > 0
    }

    pub fn apply_restrictions(&mut self, slot_id: u32, nominated_word: &String, idx_of_crossing_letter: u32) -> () 
    {
        let required = self.determine_crossing_points(slot_id, nominated_word, idx_of_crossing_letter);
        if required.is_empty() {
            return;
        }

        let mut borrowed_available_candidates = self.available_candidates.borrow_mut(); 
        for candidate in borrowed_available_candidates.iter_mut() 
        {
            let bytes = candidate.get_word().as_bytes();
            let mismatched = required.iter().any(|&(idx, letter)| bytes[idx] != letter);
            if mismatched 
            {
                candidate.record_conflict(self.slot_id, slot_id);
            }
        };
    }

    pub fn place_nominated_word_and_associated_clue(&mut self, nominated_word: String) -> () 
    {   
        // Assume a valid candidate always exists     
        if let Some(candidate) = self.available_candidates
                                     .borrow_mut()
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

    pub fn deallocate_and_discard_word(&mut self) -> () 
    {
        for cell in self.slot_cells.iter() 
        {
            cell.borrow_mut().cell = None;
        }

        // It is not clear to me whether discarding words permanently is good enough here.
        if self.selected_word != None 
        {
            self.available_discarded_candidates.push(self.selected_word.clone().unwrap());

            // Assume a valid candidate always exists   
            if let Some(candidate) = self.available_candidates
                                     .borrow_mut()
                                     .iter_mut()
                                     .find(|candidate| candidate.get_word() == &self.selected_word.clone().unwrap())
            {
                candidate.clear_assigned_slot_id();
            }
        }
        
        self.selected_word = None;
    } 
}