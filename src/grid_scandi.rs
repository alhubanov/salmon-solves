use serde::{Serialize, Deserialize};
use std::collections::{BTreeSet};
use wasm_bindgen::prelude::*;

mod clue;
mod letter;
mod gridcell;

use gridcell::GridCell;

use crate::grid_scandi::gridcell::PossibleCellState;

enum LayoutError {
    NoPossibleDomain,
    LowClueDensity,
    HighClueDensity
}

#[derive(Serialize, Deserialize)]
#[wasm_bindgen]
pub struct Grid {
    width: u32,
    height: u32,
    layout: Vec<Vec<GridCell>>,
    placed_words: BTreeSet<String>,
    clue_density: f32
}

impl Grid {
    pub fn new(width: u32, height: u32) -> Self {
        let mut layout : Vec<Vec<GridCell>> = Vec::new();

        for row_idx in 0..height {
            layout.push(Vec::new());

            for col_idx in 0..width {
                let cell : GridCell;

                if row_idx == 0 && col_idx == 0 {
                    cell = GridCell::build_starting_cell();
                } else {
                    cell = GridCell::build_generic_cell();
                }

                layout[row_idx as usize].push(cell);
            }
        }

        let placed_words = BTreeSet::new();
        let clue_density = 0.0;

        Grid { width, height, layout, placed_words, clue_density }
    }

    // TODO: rework this function properly
    pub fn print(&self) {
        for vertical_idx in 0..self.height {
            for horizontal_idx in 0..self.width {
                if self.layout[vertical_idx as usize][horizontal_idx as usize].is_undetermined() {
                    print!("# ");
                } else {
                    print!{"_ "};
                }
            }

            println!()
        }
    }

    pub fn construct_layout(&mut self) -> Result<(), LayoutError> {

        let mut num_clues = 0;

        for width_idx in 0..self.width {
            for height_idx in 0..self.height {

                if let PossibleCellState::Clue(_) = self.set_cell_state_from_remaining_possibilities(width_idx, height_idx) {
                    num_clues += 1;
                }

                if let Err(_) = self.check_neighborhood(width_idx, height_idx) {
                    // backtrack
                }

                self.check_density(num_clues)?;
            }
        }

        Ok(())
    }

    fn clear(&mut self) -> () {

    }
}
