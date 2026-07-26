use wasm_bindgen::prelude::*;
use rand::rand_core::Rng;

use crate::grid::Crossword;
use crate::grid_scandi::ScandiGrid;
use crate::grid::Grid;
use crate::grid::GenericGrid;
use crate::grid_scandi::run_stats::RunStats;

mod grid;
mod grid_simple;
pub mod grid_scandi;

mod terminal_input_utilities;
mod ui_input_utilities;

// Access point if using terminal
pub fn run() -> () {
    let mut input = String::new();

    let width = terminal_input_utilities::take_dimension_value_as_input("width", &mut input);
    let height = terminal_input_utilities::take_dimension_value_as_input("height", &mut input);

    let mut rng = rand::rng();
    let grid : ScandiGrid = build_crossword_grid_for_command_line(width, height, &mut rng);

    println!();
    println!("Here's a crossword: ");
    println!();
    
    grid.print();
}

#[inline]
pub fn build_crossword_grid_for_command_line<T : Grid>(width: u32, height: u32, rng: &mut dyn Rng) -> T 
{

    let max_depth = 
        if width < 9 || height < 9 { 1 } 
        else if width <= 12 && height <= 12 { 2 }
        else if width <= 18 && height <= 18 { 3 }
        else { 4 };
    
    let mut grid = T::initialize(width, height);
    let mut run_stats = RunStats::new();

    // Should the rng be different for each attempt?
    for _ in 0..10 
    {
        // println!("Attempt {}...", idx + 1);
        grid = T::initialize(width, height);
        if let Ok(_) = grid.construct(rng, max_depth, &mut run_stats) {
            break;
        }

        run_stats.increment_total_restarts();
    }

    run_stats.print();

    grid
}

// Access point if using frontend
// run "wasm-pack build --target web" to compile to wasm
#[wasm_bindgen]
pub fn build_crossword_grid(width: u32, height: u32, settings: JsValue) -> Result<Crossword, JsValue> 
{
    let settings: ui_input_utilities::UserSettings = serde_wasm_bindgen::from_value(settings)?;

    let max_depth = 
        if width < 9 || height < 9 { 1 } 
        else if width <= 14 && height <= 14 { 2 }
        else if width <= 20 && height <= 20 { 3 }
        else { 5 };

    let mut rng = rand::rng();
    let mut grid = GenericGrid::initialize(settings.get_grid_type(), width, height);
    let mut run_stats = RunStats::new();
    
    // Should the rng be the same for all attempts?
    for _ in 0..10 
    {
        grid = GenericGrid::initialize(settings.get_grid_type(), width, height);
        if let Ok(_) = grid.construct(&mut rng, max_depth, &mut run_stats) 
        {
            break;
        }

        run_stats.increment_total_restarts();
    }

    return Ok(Crossword::new(grid));
}   