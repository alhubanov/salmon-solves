use serde::{Serialize, Deserialize};

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, Copy, Clone)]
pub enum SlotDirection {
    Down,
    DownOnRightSide,
    Right,
    RightOnBottomSide
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct Clue {
    hint: String
}
