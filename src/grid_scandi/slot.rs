use std::collections::{BTreeSet};
use serde::{Serialize, Deserialize};

use std::rc::Rc;
use std::cell::RefCell;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, Copy, Clone)]
pub enum SlotDirection {
    Down,
    DownOnRightSide,
    Right,
    RightOnBottomSide
}

pub struct Slot {
    available_words: Rc<RefCell<BTreeSet<String>>>,
    selected_word: Option<String>,
    // using coords here instead of a &GridCell because Wasm can't handle lifetime parameters
    associated_clue_cell_height_coord: u32,
    associated_clue_cell_width_coord: u32,
    slot_direction: SlotDirection 
}

impl Slot {
    pub fn new(
        available_words: Rc<RefCell<BTreeSet<String>>>, 
        associated_clue_cell_height_coord: u32, 
        associated_clue_cell_width_coord: u32, 
        slot_direction: SlotDirection) 
    -> Self {

        let selected_word = None;

        Self { available_words, selected_word, associated_clue_cell_height_coord, associated_clue_cell_width_coord, slot_direction }
    }
}