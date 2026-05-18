use serde::{Serialize, Deserialize};
use std::{collections::BTreeSet};

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, Copy, Clone)]
pub enum SlotDirection {
    Down,
    DownOnRightSide,
    Right,
    RightOnBottomSide
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
struct AssociatedAnswer {
    answer: String,
    direction: SlotDirection,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct Clue {
    associated_answers: BTreeSet<AssociatedAnswer>
}
