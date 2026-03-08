use std::io;

pub fn run() {
    let mut input = String::new();

    let width = take_dimension_value_as_input("width", &mut input);
    let height = take_dimension_value_as_input("height", &mut input);

    println!("Width: {}", width);
    println!("Height: {}", height);
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