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

pub struct Grid {
    width: u32,
    height: u32,
    layout: Vec<Vec<GridCell>>
}

impl Grid {
    pub fn new(width: u32, height: u32) -> Self {
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