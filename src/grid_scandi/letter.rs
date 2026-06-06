#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Letter {
    cell_value: char,
}

impl Letter {
    pub fn new(cell_value: char) -> Self {
        Letter { cell_value }
    }

    pub fn get_cell_value(&self) -> char {
        self.cell_value
    }
}