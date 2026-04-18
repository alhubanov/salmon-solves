use std::io;

pub fn take_dimension_value_as_input(dimension: &str, input_container: &mut String) -> u32 {

    loop {
        println!("\nPlease input the {} of your desired crossword puzzle in number of squares: ", dimension);

        input_container.clear();

        if let Err(_) = io::stdin().read_line(input_container) {
            handle_input_error(InputError::CouldNotRead(dimension));
            continue;
        }

        let value = input_container.trim().parse::<u32>();

        match value {
            Ok(inner_val) if inner_val >= 3 => {
                return inner_val;
            },
            Ok(_) => {
                handle_input_error(InputError::TooSmall(dimension));
                continue
            },
            Err(_) => {
                handle_input_error(InputError::NotParsable(dimension));
                continue
            }
        }
    };
}

enum InputError<'a> {
    CouldNotRead(&'a str),
    TooSmall(&'a str),
    NotParsable(&'a str),
}

fn handle_input_error(state: InputError) {
    let msg = match state {
        InputError::CouldNotRead(dimension) => format!("Error: Failed to take input for {}.", dimension),
        InputError::TooSmall(dimension) => format!("Error: Provided {} is too small.", dimension),
        InputError::NotParsable(dimension) => format!("Error: Failed to parse provided {} into u32.", dimension)
    };

    print_error_message_for_bad_input(&msg);
}

fn print_error_message_for_bad_input(custom_message: &str) -> () {
    println!("\n{}", custom_message);
    println!("Please provide a valid integer larger than 2.");
}