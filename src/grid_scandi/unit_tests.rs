use crate::grid_scandi::gridcell::CellType::Letter;
use crate::grid_scandi::gridcell::CellType::Clue;

use super::*;
use rand;
// use rand::rand_core::SeedableRng;
// use chacha20::ChaCha8Rng;

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
//     let grid1 = 
//     {
//         let mut rng = ChaCha8Rng::seed_from_u64(2);
//         let mut grid1 = ScandiGrid::initialize(5, 5);
//         grid1.construct(&mut rng).ok();
//         grid1.layout
//     };

//     let grid2 = 
//     {
//         let mut rng = ChaCha8Rng::seed_from_u64(2);
//         let mut grid2 = ScandiGrid::initialize(5, 5);
//         grid2.construct(&mut rng).ok();
//         grid2.layout
//     };

//     assert_eq!(grid1, grid2);
// }

#[test]
fn grid_is_valid()
{
    let mut rng = rand::rng();
    let mut grid = ScandiGrid::initialize(6, 6);
    grid.construct(&mut rng).ok();

    let words_vec : Vec<&str> = WORDS.lines().collect();
    for slot in grid.word_slots 
    {
        assert!(slot.get_selected_word().is_some());

        let mut reconstructed_word = String::from("");
        for cell in slot.get_cells()
        {
            let borrowed_cell = cell.borrow();
            match borrowed_cell.cell.as_ref().unwrap() 
            {
                Letter(letter) => { reconstructed_word.push(letter.get_cell_value()); },
                Clue(_) => panic!("Test failed at clue."),
            }
        }
        assert_eq!(reconstructed_word, *slot.get_selected_word().as_ref().unwrap());
        assert!(words_vec.contains(&slot.get_selected_word().as_ref().unwrap().as_str()));
    }
}
