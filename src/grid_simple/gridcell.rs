use serde::Serialize;

#[derive(PartialEq, Debug, Serialize)]
pub enum CellState {
    Filled,
    NotFilled,
    TentativelyFilled
}

#[derive(PartialEq, Serialize)]
pub enum CrossingState {
    Is,
    IsTentatively,
    IsNot
}

#[derive(Serialize)]
pub struct GridCell {
    pub cell_value: char,
    pub cell_state: CellState,
    pub is_a_crossing: CrossingState
}

impl GridCell {
    pub fn new() -> Self {
        Self { cell_value: '_', cell_state: CellState::NotFilled, is_a_crossing: CrossingState::IsNot }
    }

    pub fn reset(&mut self) {
        self.cell_value = '_';
        self.cell_state = CellState::NotFilled;
        self.reset_crossing_state();
    }

    pub fn reset_crossing_state(&mut self) {
        self.is_a_crossing = CrossingState::IsNot;
    }
}
