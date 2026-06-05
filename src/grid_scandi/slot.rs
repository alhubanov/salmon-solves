use std::collections::{BTreeSet};
use serde::{Serialize, Deserialize};

use std::rc::Rc;
use std::cell::RefCell;
use std::cell::Ref;

use crate::grid_scandi::gridcell::GridCell;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, Copy, Clone)]
pub enum SlotDirection {
    Down,
    DownOnRightSide,
    Right,
    RightOnBottomSide
}

pub struct Slot {
    selected_word: Option<String>,
    available_words: Rc<RefCell<BTreeSet<String>>>,
    associated_clue_cell: Rc<RefCell<GridCell>>,
    slot_length: u32,
    slot_cells: Vec<Rc<RefCell<GridCell>>>,
    slot_direction: SlotDirection 
}

impl Slot {
    pub fn new(
        slot_length: u32,
        available_words: Rc<RefCell<BTreeSet<String>>>, 
        associated_clue_cell: Rc<RefCell<GridCell>>,
        slot_cells: Vec<Rc<RefCell<GridCell>>>,
        slot_direction: SlotDirection) 
    -> Self {

        let selected_word = None;

        Self { 
            selected_word, 
            available_words, 
            associated_clue_cell,
            slot_length, 
            slot_cells,
            slot_direction 
        }
    }

    pub fn get_available_word_set(&self) -> Ref<'_, BTreeSet<String>> {
        self.available_words.borrow()
    }
}