use std::io;
use std::fs;

enum CellState {
    Filled,
    NotFilled
}

struct GridCell {
    cell_value: char,
    cell_state: CellState
}

impl GridCell {
    fn new() -> Self {
        Self { cell_value: '_', cell_state: CellState::NotFilled }
    }
}

struct Grid {
    width: u32,
    height: u32,
    layout: Vec<Vec<GridCell>>
}

impl Grid {
    fn new(width: u32, height: u32) -> Self {
        let mut layout : Vec<Vec<GridCell>> = Vec::new();

        for row_idx in 0..height {
            layout.push(Vec::new());

            for _ in 0..width {
                let cell = GridCell::new();
                layout[row_idx as usize].push(cell);
            }
        }

        Grid { width, height, layout }
    }
}


pub fn run() -> () {
    let mut input = String::new();

    let width = take_dimension_value_as_input("width", &mut input);
    let height = take_dimension_value_as_input("height", &mut input);

    let grid = Grid::new(width, height);
    let words = match fs::read_to_string("./word_files/common_english_words_long.txt") {
        Ok(file_content) => file_content, 
        Err(_) => {
            println!("Internal Error: Could not read the source file containing all the words. Program will exit now.");
            return;
        }
    };

    println!("Width: {}", width);
    println!("Height: {}", height);
    println!("Words: {}", words);
}

fn take_dimension_value_as_input(dimension: &str, input_container: &mut String) -> u32 {

    loop {
        println!("Please input the {} of your desired crossword puzzle in number of squares: ", dimension);

        if let Err(_) = io::stdin().read_line(input_container) {
            println!("Error: Failed to take input for {}.", dimension);
            println!("Please provide a valid non-negative integer.");
            continue;
        }

        let value = input_container.trim().parse::<u32>();

        match value {
            Ok(inner_val) => { 
                input_container.clear();
                println!();
                return inner_val; 
            },
            Err(_) => {
                input_container.clear();
                println!("Error: Failed to parse provided {} into u32.", dimension);
                println!("Please provide a valid non-negative integer.");
                continue
            }
        }

    };
}