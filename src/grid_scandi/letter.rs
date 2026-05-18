use serde::{Serialize, Deserialize};

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct Letter {
    cell_value: char,
}