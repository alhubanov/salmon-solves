use crate::grid_scandi::gridcell::CellType::Letter;
use crate::grid_scandi::gridcell::CellType::Clue;
use crate::grid_scandi::dictionary;

use super::*;
// use chacha20::ChaCha8Rng;
use rand;

static WORDS: &str = include_str!("../../word_files/oewn-answer-clue-deduped-clean.tsv");

// #[test]
// fn layout_first_row_and_column() {
//     let mut grid: ScandiGrid = ScandiGrid::initialize(5, 5);
//     let mut rng = rand::rng();
    
//     for _ in 0..10 
//     {
//         if let Ok(_) = grid.construct(&mut rng) {
//             break;
//         }
//     }

//     test_helpers::assert_correct_first_row(&grid);
//     test_helpers::assert_correct_first_col(&grid);
// }

// #[test]
// fn construct_is_deterministic_if_rng_is_seeded() 
// {
//     let dictionary = Rc::new(RefCell::new(dictionary::Dictionary::build(WORDS)));
//     let mut run_stats = RunStats::new();
//     let grid1 = 
//     {
//         let mut rng = ChaCha8Rng::seed_from_u64(14);
//         let mut grid1 = ScandiGrid::initialize(12, 12, Rc::clone(&dictionary));
//         grid1.construct(&mut rng, 2, &mut run_stats).ok();
//         grid1.layout
//     };

//     let mut run_stats = RunStats::new();
//     let grid2 = 
//     {
//         let mut rng = ChaCha8Rng::seed_from_u64(14);
//         let mut grid2 = ScandiGrid::initialize(12, 12, Rc::clone(&dictionary));
//         grid2.construct(&mut rng,2, &mut run_stats).ok();
//         grid2.layout
//     };

//     assert_eq!(grid1, grid2);
// }

#[test]
fn grid_is_valid()
{

    let dictionary = Rc::new(RefCell::new(dictionary::Dictionary::build(WORDS)));
    // Testing with higher sizes can encounter "backtracking threshold exceeded", which causes the test to fail.
    // The "construct" function is not the right function to test here.
    for size in 3..10
    {
        let mut rng = rand::rng();
        let mut grid = ScandiGrid::initialize(size, size, Rc::clone(&dictionary));
        let mut run_stats = RunStats::new();

        if let Err(err) = grid.construct(&mut rng, 2, &mut run_stats)
        {
            println!("Err: {:?}", err);
        }

        grid.print();
        println!();

        let words_vec : Vec<&str> = WORDS.lines().map(|line| line.split_once('\t').unwrap().0).collect();
        for slot in grid.word_slots 
        {
            assert!(slot.get_selected_word().is_some());

            let mut reconstructed_word = String::from("");
            for cell in slot.get_cells()
            {
                let borrowed_cell = cell.borrow();
                match borrowed_cell.cell.as_ref().unwrap() 
                {
                    Letter(letter, _) => { reconstructed_word.push(letter.unwrap()); },
                    Clue(_) => panic!("Test failed at clue."),
                }
            }
            assert_eq!(reconstructed_word, *slot.get_selected_word().as_ref().unwrap());
            assert!(words_vec.contains(&slot.get_selected_word().as_ref().unwrap().as_str()));
        }
    }
}
