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

pub struct Slot {
    slot_id: u32,
    selected_word: Option<String>,
    suitable_words: Rc<RefCell<BTreeSet<String>>>,
    suitable_discarded_words: BTreeSet<String>, 
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

        Self { 
            slot_id,
            selected_word, 
            suitable_words, 
            suitable_discarded_words,
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

    pub fn select_word_to_place(&mut self) -> Result<(), (LayoutError, BTreeSet<u32>)> {

        // println!("Selecting word for slot id {}", self.slot_id);

        let mut unsuitable_words : BTreeSet<String> = BTreeSet::new();
        let mut crossing_slot_ids : BTreeSet<u32> = BTreeSet::new();

        for cell in self.slot_cells.iter() {
            if let Some(CellType::Letter(_)) = cell.borrow().cell.as_ref() {
                crossing_slot_ids.extend(cell.borrow().slot_ids.as_ref().unwrap().iter().filter(|elem| *elem != &self.slot_id));
            }
        }

        for word in self.suitable_words.borrow().iter() 
        {
            let chars: Vec<char> = word.chars().collect();
            for (idx, cell) in self.slot_cells.iter().enumerate() 
            {
                if let Some(CellType::Letter(letter)) = cell.borrow().cell.as_ref() 
                {
                    if letter.get_cell_value() != chars[idx] 
                    {
                        unsuitable_words.insert(word.clone());
                        break;
                    }
                }
            }
        }

        let mut rng = rand::rng();
        let sampled_word = self.suitable_words
                            .borrow()
                            .iter()
                            .filter(|word| !unsuitable_words.contains(*word) && !self.suitable_discarded_words.contains(*word))
                            .choose(&mut rng)
                            .ok_or((LayoutError::NoPossibleDomain, crossing_slot_ids))?
                            .clone();

        self.selected_word = Some(sampled_word);

        Ok(())
    }

    pub fn allocate_word(&mut self) -> Result<(), (LayoutError, BTreeSet<u32>)> {

        self.select_word_to_place()?;

        for (idx, cell) in self.slot_cells.iter().enumerate() {
            if cell.borrow().cell.is_none() {
                cell.borrow_mut().cell = Some(CellType::Letter(letter::Letter::new(self.selected_word.as_ref().unwrap().chars().nth(idx).unwrap())));
            }
        }

        self.suitable_words.borrow_mut().remove(&(self.selected_word.clone().unwrap()));

        Ok(())
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