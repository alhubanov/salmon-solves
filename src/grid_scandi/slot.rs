use std::collections::BTreeMap;
use std::collections::{BTreeSet};
use serde::{Serialize, Deserialize};
use rand::prelude::*;

use std::rc::Rc;
use std::cell::RefCell;
use std::cell::Ref;

use crate::grid_scandi::LayoutError;
use crate::grid_scandi::gridcell::GridCell;

use crate::grid_scandi::gridcell::CellType;
use crate::grid_scandi::letter;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, Copy, Clone)]
pub enum SlotDirection {
    Down,
    DownOnRightSide,
    Right,
    RightOnBottomSide
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Slot {
    slot_id: u32,
    selected_word: Option<String>,
    suitable_words: Rc<RefCell<BTreeSet<String>>>,
    suitable_discarded_words: BTreeSet<String>, 
    unsuitable_words_per_crossing: BTreeMap<u32, BTreeSet<String>>,
    temporary_unsuitable_words: BTreeSet<String>,
    // associated_clue_cell: Rc<RefCell<GridCell>>,
    // slot_length: u32,
    slot_cells: Vec<Rc<RefCell<GridCell>>>,
    // slot_direction: SlotDirection 
}

impl Slot {
    pub fn new(
        slot_id: u32,
        // slot_length: u32,
        suitable_words: Rc<RefCell<BTreeSet<String>>>,
        // associated_clue_cell: Rc<RefCell<GridCell>>,
        slot_cells: Vec<Rc<RefCell<GridCell>>>,
        // slot_direction: SlotDirection
        ) 
    -> Self {

        let selected_word = None;
        let suitable_discarded_words = BTreeSet::new();
        let unsuitable_words_per_crossing = BTreeMap::new();
        let temporary_unsuitable_words = BTreeSet::new();

        Self { 
            slot_id,
            selected_word, 
            suitable_words, 
            suitable_discarded_words,
            unsuitable_words_per_crossing,
            temporary_unsuitable_words,
            // associated_clue_cell,
            // slot_length, 
            slot_cells,
            // slot_direction 
        }
    }

    pub fn get_suitable_word_set(&self) -> Ref<'_, BTreeSet<String>> {
        self.suitable_words.borrow()
    }

    pub fn get_slot_id(&self) -> u32 {
        self.slot_id
    }

    pub fn get_crossing_slot_ids(&self) -> BTreeSet<u32> {

        let mut crossing_slot_ids : BTreeSet<u32> = BTreeSet::new();
        for cell in self.slot_cells.iter() {
            if let Some(CellType::Letter(_)) = cell.borrow().cell.as_ref() {
                crossing_slot_ids.extend(cell.borrow().slot_ids.as_ref().unwrap().iter().filter(|elem| *elem != &self.slot_id));
            }
        }

        crossing_slot_ids
    }

    pub fn has_placed_word(&self) -> bool {
        self.selected_word.is_some()
    }

    fn check_if_placement_is_possible(&self, sampled_word: &String) -> Result<(), LayoutError> {

        let letters : Vec<char> = sampled_word.chars().collect();
        for idx in 0..self.slot_cells.len() 
        {
            match &self.slot_cells[idx].borrow().cell 
            {
                Some(CellType::Letter(letter)) => { if letter.get_cell_value() != letters[idx] { return Err(LayoutError::CannotPlaceWord); }},
                _  => continue
            }
        }

        Ok(())
    }

    pub fn nominate_word(&mut self) -> Result<String, LayoutError> {
        
        let mut rng = rand::rng();
        let mut sampled_word : String;

        loop {
            sampled_word = self.suitable_words
                            .borrow()
                            .iter()
                            .filter(|word| !self.unsuitable_words_per_crossing.iter().any(|(_, set)| set.contains(*word)) && !self.suitable_discarded_words.contains(*word))
                            .choose(&mut rng)
                            .ok_or(LayoutError::NoPossibleDomain)?
                            .clone();

            if let Ok(_) = self.check_if_placement_is_possible(&sampled_word) {
                break;
            }
        }

        return Ok(sampled_word);
    }

    pub fn has_possibilities_remaining(&mut self, slot_id: u32, nominated_word: &String) -> bool {

        self.temporary_unsuitable_words.clear();

        let mut num_unsuitable_words_found = 0;
        for curr_idx in 0..self.slot_cells.len() 
        {
            if self.slot_cells[curr_idx].borrow_mut().slot_ids.as_ref().is_some_and(|ids| ids.contains(&slot_id)) {
                let char_of_interest = nominated_word.chars().nth(curr_idx).expect("Asked to nominate a longer word than slot length.");
                
                for word in self.suitable_words.borrow().iter() {
                    if word.chars().nth(curr_idx).expect("There is a 'suitable' word of wrong length.") != char_of_interest {
                        self.temporary_unsuitable_words.insert(word.clone());
                        num_unsuitable_words_found += 1;
                    }
                }
            }
        };

        num_unsuitable_words_found < self.suitable_words.borrow().len()
    }

    pub fn remove_unsuitable_words_related_to_slot_id(&mut self, slot_id: u32) -> () {
        self.unsuitable_words_per_crossing.remove(&slot_id);
    }

    pub fn apply_restrictions(&mut self, slot_id: u32) -> () {
        self.unsuitable_words_per_crossing.insert(slot_id, std::mem::take(&mut self.temporary_unsuitable_words));
    }

    pub fn place_nominated_word(&mut self, nominated_word: String) -> () 
    {
        self.selected_word = Some(nominated_word);
        for (idx, cell) in self.slot_cells.iter().enumerate() {
            if cell.borrow().cell.is_none() {
                cell.borrow_mut().cell = Some(CellType::Letter(letter::Letter::new(self.selected_word.as_ref().unwrap().chars().nth(idx).unwrap())));
            }
        }

        self.suitable_words.borrow_mut().remove(&(self.selected_word.clone().unwrap()));
    }

    pub fn deallocate_and_discard_word(&mut self) -> () 
    {
        for cell in self.slot_cells.iter() {
            cell.borrow_mut().cell = None;
        }

        // It is not clear to me whether discarding words permanently is good enough here.
        self.suitable_discarded_words.insert(self.selected_word.clone().unwrap());
        self.suitable_words.borrow_mut().insert(self.selected_word.clone().unwrap());
        self.selected_word = None;
    } 
}