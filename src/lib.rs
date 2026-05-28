use wasm_bindgen::prelude::*;

use crate::grid_simple::SimpleGrid;
use crate::grid_scandi::ScandiGrid;
use crate::grid::Grid;

mod grid;
mod grid_scandi;
mod grid_simple;
mod input_utilities;

pub fn run() -> () {
    let mut input = String::new();

    let width = input_utilities::take_dimension_value_as_input("width", &mut input);
    let height = input_utilities::take_dimension_value_as_input("height", &mut input);

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
pub fn build_crossword_grid(width: u32, height: u32) -> Result<JsValue, serde_wasm_bindgen::Error> {

    let mut grid = SimpleGrid::initialize(width, height);
    grid.construct();

    return serde_wasm_bindgen::to_value(&grid);
} 