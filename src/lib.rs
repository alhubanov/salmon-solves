use rand::{self, seq::SliceRandom};
use wasm_bindgen::prelude::*;

use crate::grid_simple::Grid;
use crate::grid_scandi::Grid as ScandiGrid;

pub mod grid_scandi;
pub mod grid_simple;
pub mod input_utilities;

static WORDS: &str = include_str!("../word_files/common_english_words_long.txt");

pub fn run() -> () {
    let mut input = String::new();

    let width = input_utilities::take_dimension_value_as_input("width", &mut input);
    let height = input_utilities::take_dimension_value_as_input("height", &mut input);

    let grid = match build_scandinavian_grid_for_command_line(width, height) {
        Ok(res) => res,
        Err(s) => { 
            println!("{}", s); 
            return; 
        } 
    };

    println!();
    println!("Here's a crossword: ");
    println!();
    
    grid.print();
}

pub fn build_scandinavian_grid_for_command_line(width: u32, height: u32) -> Result<ScandiGrid, String> {
    let mut grid = ScandiGrid::new(width, height);
    
    if let Err(_) = grid.construct_layout() {
        return Err("Failed construction".to_string());
    }

    return Ok(grid);
}

pub fn build_crossword_grid_for_command_line(width: u32, height: u32) -> Result<Grid, String> {

    let mut grid = Grid::new(width, height);
    let mut words_vec : Vec<&str> = WORDS.lines().collect();

    let mut rng = rand::rng();
    words_vec.shuffle(&mut rng);

    for word in words_vec {
        if let Err(_) = grid.place_word(&word.to_owned()) {
            continue;
        }
    }

    return Ok(grid);
} 

#[wasm_bindgen]
pub fn build_crossword_grid(width: u32, height: u32) -> Result<JsValue, String> {

    let mut grid = Grid::new(width, height);
    let mut words_vec : Vec<&str> = WORDS.lines().collect();

    let mut rng = rand::rng();
    words_vec.shuffle(&mut rng);

    for word in words_vec {
        if let Err(_) = grid.place_word(&word.to_owned()) {
            continue;
        }
    }

    return Ok(serde_wasm_bindgen::to_value(&grid).unwrap());
} 