use rand::{self, seq::SliceRandom};
use wasm_bindgen::prelude::*;

use crate::grid::Grid;

pub mod grid;
pub mod input_utilities;

static WORDS: &str = include_str!("../word_files/common_english_words_long.txt");

pub fn run() -> () {
    let mut input = String::new();

    let width = input_utilities::take_dimension_value_as_input("width", &mut input);
    let height = input_utilities::take_dimension_value_as_input("height", &mut input);

    let grid = match build_crossword_grid(width, height) {
        Ok(res) => res,
        Err(s) => { 
            println!("{}", s); 
            return; 
        } 
    };

    // println!();
    // println!("Here's a crossword: ");
    // println!();
    
    // grid.print();
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