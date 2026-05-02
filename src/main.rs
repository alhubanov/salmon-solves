use crossy::run;

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
    println!("  * * * * *     * * * * *     * * * * *     * * * * *     * * * * *     *             *   ");
    println!("*               *       *     *       *     *             *               *         *     ");
    println!("*               *       *     *       *     *             *                 *     *       ");
    println!("*               *       *     *       *     *             *                   * *         ");
    println!("*               * * * * *     *       *     * * * * *     * * * * *            *          ");
    println!("*               * *           *       *             *             *            *          ");
    println!("*               *   *         *       *             *             *            *          ");
    println!("*               *     *       *       *             *             *            *          ");
    println!("  * * * * *     *       *     * * * * *     * * * * *     * * * * *            *          ");
}
