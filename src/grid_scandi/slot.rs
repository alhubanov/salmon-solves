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

    pub fn get_crossings(&self) -> BTreeSet<(u32, u32)> {

        let mut crossing_slot_ids : BTreeSet<(u32, u32)> = BTreeSet::new();
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

    pub fn has_placed_word(&self) -> bool {
        self.selected_word.is_some()
    }

    pub fn nominate_word(&mut self, already_attempted_words: &BTreeSet<String>) -> Result<String, LayoutError> {
        
        let mut rng = rand::rng();
        let borrowed_suitable_words = self.suitable_words.borrow();
        let candidate_words: Vec<&String> = borrowed_suitable_words
                                                .iter()
                                                .filter(|word| !self.unsuitable_words_per_crossing.iter().any(|(_, set)| set.contains(*word)) && 
                                                                !self.suitable_discarded_words.contains(*word) && 
                                                                !already_attempted_words.contains(*word))
                                                .collect();

        if candidate_words.is_empty() { return Err(LayoutError::NoPossibleDomain); }
        let idx = rng.random_range(0..candidate_words.len());
        let sampled_word = candidate_words[idx].clone();

        return Ok(sampled_word);
    }

    pub fn find_temporary_unsuitable_words(&mut self, slot_id: u32, nominated_word: &String, idx_of_crossing_letter: u32) -> () 
    {
        println!("Reducing possibilities for slot {} because of crossing nominate_word {} from slot {}", self.slot_id, nominated_word, slot_id);
        self.temporary_unsuitable_words.clear();

        let required: Vec<(usize, u8)> = self.slot_cells
                                            .iter()
                                            .enumerate()
                                            .filter_map(|(idx, cell)| {
                                                if cell.borrow().slot_ids.as_ref().is_some_and(|ids| ids.contains(&slot_id)) {
                                                    println!("Crossing point in slot is idx {}, with letter {}", idx, nominated_word.as_bytes()[idx_of_crossing_letter as usize]);
                                                    Some((idx, nominated_word.as_bytes()[idx_of_crossing_letter as usize]))
                                                } else {
                                                    None
                                                }
                                            })
                                            .collect();

        if required.is_empty() {
            return;
        }

        let suitable_words = self.suitable_words.borrow(); 
        let suitable_words_left : BTreeSet<&String> = suitable_words.iter().filter(|w| !self.unsuitable_words_per_crossing.iter().any(|(_, set)| set.contains(*w))).collect();

        println!("About to try iterating through suitable.");
        for word in suitable_words_left.iter() 
        {
            let bytes = word.as_bytes();
            let mismatched = required.iter().any(|&(idx, letter)| bytes[idx] != letter);
            if mismatched 
            {
                self.temporary_unsuitable_words.insert(word.to_string());
            }
        };

        println!("Found {} temp unsuitable words", self.temporary_unsuitable_words.len());
    }

    pub fn has_possibilities_remaining(&mut self, slot_id: u32, nominated_word: &String, idx_of_crossing_letter: u32) -> bool 
    {
        self.find_temporary_unsuitable_words(slot_id, nominated_word, idx_of_crossing_letter);

        let suitable_words = self.suitable_words.borrow(); 
        let suitable_words_left : BTreeSet<&String> = suitable_words.iter().filter(|w| !self.unsuitable_words_per_crossing.iter().any(|(_, set)| set.contains(*w))).collect();

        self.temporary_unsuitable_words.len() < suitable_words_left.len()
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

        if self.selected_word != None {
            self.suitable_discarded_words.insert(self.selected_word.clone().unwrap());
            self.suitable_words.borrow_mut().insert(self.selected_word.clone().unwrap());
        }
        
        self.selected_word = None;
    } 
}