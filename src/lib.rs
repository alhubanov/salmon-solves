use std::fs;

use crate::grid::Grid;

pub mod grid;
pub mod input_utilities;

pub fn run() -> () {
    let mut input = String::new();

    let width = input_utilities::take_dimension_value_as_input("width", &mut input);
    let height = input_utilities::take_dimension_value_as_input("height", &mut input);

    let mut grid = Grid::new(width, height);
    let words = match fs::read_to_string("./word_files/common_english_words_long.txt") {
        Ok(file_content) => file_content, 
        Err(_) => {
            println!("Internal Error: Could not read the source file containing all the words. Program will exit now.");
            return;
        }
    };

    for word in words.lines() {
        if let Err(_) = grid.place_word(&word.to_owned()) {
            continue;
        }
    }

    println!();
    println!("Here's a crossword: ");
    println!();
    
    grid.print();
}