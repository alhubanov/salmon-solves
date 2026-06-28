use wasm_bindgen::prelude::*;
use rand::{prelude::*};

use crate::{grid_scandi::ScandiGrid, grid_simple::SimpleGrid, ui_input_utilities::GridType};

pub enum LayoutError {
    NoPossibleDomainAfterRecursion,
    NoPossibleDomain,
    CannotPlaceWord,
    IsNotAValidSlot,
    // LowClueDensity,
    // HighClueDensity,
    // IsolatedLetter,
    CannotEnsureAClearWordSlot
}

pub trait Grid {
    fn initialize(width: u32, height: u32) -> Self;
    fn construct(&mut self, rng: &mut dyn Rng) -> Result<(), LayoutError>;
    fn print(&self) -> ();
}

pub enum GenericGrid {
    Simple(SimpleGrid),
    Scandi(ScandiGrid)
}

impl GenericGrid {
    pub fn initialize(grid_type: &GridType, width: u32, height: u32) -> Self {
        match grid_type {
            GridType::Scandi => GenericGrid::Scandi(ScandiGrid::initialize(width, height)),
            GridType::Simple => GenericGrid::Simple(SimpleGrid::initialize(width, height)),
        }
    }
}

impl Grid for GenericGrid  {
    fn initialize(_width: u32, _height: u32) -> Self {
        panic!("Use GenericGrid::initialize(grid_type, width, height) instead.")
    }

    fn construct(&mut self, rng: &mut dyn Rng) -> Result<(), LayoutError> {
        match self {
            Self::Scandi(scandi_grid) => scandi_grid.construct(rng),
            Self::Simple(simple_grid) => simple_grid.construct(rng)
        }
    }

    fn print(&self) -> () {
        match self {
            Self::Scandi(scandi_grid) => scandi_grid.print(),
            Self::Simple(simple_grid) => simple_grid.print()
        }
    }
}

#[wasm_bindgen]
pub struct Crossword {
    _constructed_grid: GenericGrid
}

impl Crossword {
    pub fn new(_constructed_grid: GenericGrid) -> Self {
        Self { _constructed_grid }
    }
}