use crossy::run;
use termion::{color, style};

fn main() {
    print_header();
    run();
}

fn print_header() {
    println!();
    print_title_art();
    println!();
    println!("Welcome to Crossy! Follow the instructions to create your own crossword puzzle.");
    println!();
}

fn print_title_art() {
    print!("{}{}", color::Fg(color::Blue), style::Bold);
    println!("  * * * * *     * * * * *     * * * * *     * * * * *     * * * * *     *             *   ");
    println!("*               *       *     *       *     *             *               *         *     ");
    println!("*               *       *     *       *     *             *                 *     *       ");
    println!("*               *       *     *       *     *             *                   * *         ");
    println!("*               * * * * *     *       *     * * * * *     * * * * *            *          ");
    println!("*               * *           *       *             *             *            *          ");
    println!("*               *   *         *       *             *             *            *          ");
    println!("*               *     *       *       *             *             *            *          ");
    println!("  * * * * *     *       *     * * * * *     * * * * *     * * * * *            *          ");
    print!("{}", style::Reset);
}
