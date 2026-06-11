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
    associated_clue_cell: Rc<RefCell<GridCell>>,
    slot_length: u32,
    slot_cells: Vec<Rc<RefCell<GridCell>>>,
    slot_direction: SlotDirection 
}

impl Slot {
    pub fn new(
        slot_id: u32,
        slot_length: u32,
        suitable_words: Rc<RefCell<BTreeSet<String>>>,
        associated_clue_cell: Rc<RefCell<GridCell>>,
        slot_cells: Vec<Rc<RefCell<GridCell>>>,
        slot_direction: SlotDirection) 
    -> Self {

        println!("Creating slot with slot id {}", slot_id);
        println!("Slot len is {}", slot_length);
        println!("Num of slot cells is {}", slot_cells.len());

        let selected_word = None;
        let suitable_discarded_words = BTreeSet::new();

        Self { 
            slot_id,
            selected_word, 
            suitable_words, 
            suitable_discarded_words,
            associated_clue_cell,
            slot_length, 
            slot_cells,
            slot_direction 
        }
    }

    pub fn get_suitable_word_set(&self) -> Ref<'_, BTreeSet<String>> {
        self.suitable_words.borrow()
    }

    pub fn get_slot_id(&self) -> u32 {
        self.slot_id
    }

    pub fn select_word_to_place(&mut self) -> Result<(), (LayoutError, BTreeSet<u32>)> {

        let mut unsuitable_words : BTreeSet<String> = BTreeSet::new();
        let mut crossing_slot_ids : BTreeSet<u32> = BTreeSet::new();

        for (idx, cell) in self.slot_cells.iter().enumerate() {

            println!("Iterating through cell {}", idx);
            if cell.borrow().cell.is_none() {
                println!("Cell is none still");
                continue;
            }

            for word in self.suitable_words.borrow().iter() {
                match cell.borrow().cell.as_ref().unwrap() {
                    CellType::Letter(letter) => {
                        crossing_slot_ids.insert(cell.borrow().slot_id.unwrap());

                        if letter.get_cell_value() != word.chars().nth(idx).unwrap() {
                            unsuitable_words.insert(word.clone());
                        }
                    },
                    CellType::Clue(_) => panic!("Impossible slot configuration.")
                };
            }
        }

        let unapplicable_words : BTreeSet<String> = unsuitable_words.union(&self.suitable_discarded_words).cloned().collect();

        let suitable_words = self.suitable_words.borrow();
        let mut rng = rand::rng();
        let sampled_word = suitable_words.difference(&unapplicable_words).choose(&mut rng).ok_or((LayoutError::NoPossibleDomain, crossing_slot_ids))?.clone();

        self.selected_word = Some(sampled_word);

        println!("1. Selected word is {}", self.selected_word.as_ref().unwrap());


        Ok(())
    }

    pub fn allocate_word(&mut self) -> Result<(), (LayoutError, BTreeSet<u32>)> {

        self.select_word_to_place()?;

        println!("2. Selected word is {}", self.selected_word.as_ref().unwrap());

        for (idx, cell) in self.slot_cells.iter().enumerate() {
            if cell.borrow().cell.is_none() {

                println!("In allocating.");
                println!("For idx {} the nth char is {:?}", idx, self.selected_word.as_ref().unwrap().chars().nth(idx));

                cell.borrow_mut().cell = Some(CellType::Letter(letter::Letter::new(self.selected_word.as_ref().unwrap().chars().nth(idx).unwrap())));
                cell.borrow_mut().slot_id = Some(self.slot_id);
            }
        }

        self.suitable_words.borrow_mut().remove(&(self.selected_word.clone().unwrap()));

        Ok(())
    }

    pub fn deallocate_and_discard_word(&mut self) -> () {

        for cell in self.slot_cells.iter() {
            if cell.borrow().cell.is_some() && cell.borrow().slot_id.unwrap() == self.slot_id {
                cell.borrow_mut().cell = None;
                cell.borrow_mut().slot_id = None;
            }
        }

        // It is not clear to me whether discarding words permanently is good enough here.
        self.suitable_discarded_words.insert(self.selected_word.clone().unwrap());
        self.suitable_words.borrow_mut().insert(self.selected_word.clone().unwrap());
        self.selected_word = None;
    } 
}