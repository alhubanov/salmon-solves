use wasm_bindgen::prelude::*;

use crate::grid_scandi::ScandiGrid;
use crate::grid::Grid;

mod grid;
mod grid_scandi;
mod grid_simple;
mod terminal_input_utilities;
mod ui_input_utilities;

pub fn run() -> () {
    let mut input = String::new();

    let width = terminal_input_utilities::take_dimension_value_as_input("width", &mut input);
    let height = terminal_input_utilities::take_dimension_value_as_input("height", &mut input);

    let grid : ScandiGrid = build_crossword_grid_for_command_line(width, height);

    println!();
    println!("Here's a crossword: ");
    println!();
    
    grid.print();
}

pub fn build_crossword_grid_for_command_line<T : Grid>(width: u32, height: u32) -> T {

    let mut grid = T::initialize(width, height);
    grid.construct();

    return grid;
} 

#[wasm_bindgen]
pub fn build_crossword_grid(width: u32, height: u32, settings: JsValue) -> Result<JsValue, serde_wasm_bindgen::Error> {
    let settings: ui_input_utilities::UserSettings = serde_wasm_bindgen::from_value(settings)?;

    let mut grid = grid::GenericGrid::initialize(settings.get_grid_type(), width, height);
    grid.construct();

    return serde_wasm_bindgen::to_value(&grid);
}   