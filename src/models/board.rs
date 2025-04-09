// src/models/board.rs
//
// Defining the Tetris Board model

use crate::views::{BoardPosition, PieceInstance};

#[derive(PartialEq)]
pub enum PlaceResult {
    PlaceOk,
    RowFilled,
    OutOfBounds,
    PlaceBad,
}

pub struct Board {
    pub width: isize,  // the overall width in cells
    pub height: isize, // the overall height in cells
    state: BoardState, // the grid state
    locked: bool,      // true when all pieces are locked
}

impl Board {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width: width as isize,
            height: height as isize,
            state: BoardState::new(width, height),
            locked: false,
        }
    }

    /************************ Piece Placement *******************************/

    // Sees if the next placement is valid
    pub fn try_place(&mut self, piece: &PieceInstance, pos: BoardPosition) -> PlaceResult {
        for &(dx, dy) in piece.cells() {
            let x = pos.x + dx;
            let y = pos.y + dy;

            if self.idx(x, y).is_none() {
                return PlaceResult::OutOfBounds;
            }

            if self.is_cell_filled(BoardPosition { x, y }) {
                return PlaceResult::PlaceBad;
            }
        }

        PlaceResult::PlaceOk
    }

    // commit a pre-validated piece
    pub fn commit_piece(&mut self, piece: &PieceInstance) {
        for &(dx, dy) in piece.cells() {
            self.fill_cell(BoardPosition {
                x: piece.position.x + dx,
                y: piece.position.y + dy,
            });
        }
    }

    pub fn get_drop_location(&self, piece: &PieceInstance) -> BoardPosition {
        let skirt = piece.typ.skirt(piece.rot_idx);

        // Calculate grid min/max x
        let (min_dx, max_dx) = piece.typ.minmax_x(piece.rot_idx);

        // Find the drop height
        let mut max_required_y = 0;

        // iterate over each column that the piece occupies
        for x_offset in 0..=(max_dx - min_dx) {
            // convert relative_x to board x, accounting for how skirt index is
            // relative to min_x
            let board_x = piece.position.x + min_dx + x_offset;

            // check if OOB
            if board_x < 0 || board_x >= self.width {
                continue;
            }

            let skirt_val = skirt[x_offset as usize];

            // get this column's height
            let col_height = self.col_score(board_x).unwrap_or(0);

            // calculate the required valid y value for this column
            let required_y = if col_height == 0 {
                0 - skirt_val // place at grid bottom
            } else {
                col_height + 1 - skirt_val // place above highest piece
            };

            if required_y > max_required_y {
                max_required_y = required_y;
            }
        }

        // 0 is the minimum y value
        let final_y = std::cmp::max(0, max_required_y);

        BoardPosition {
            x: piece.position.x,
            y: final_y,
        }
    }

    pub fn is_cell_filled(&self, pos: BoardPosition) -> bool {
        match self.idx(pos.x, pos.y) {
            Some(inx) => self.state.grid[inx],
            None => false,
        }
    }

    fn fill_cell(&mut self, pos: BoardPosition) {
        if let Some(idx) = self.idx(pos.x, pos.y) {
            // Update grid
            self.state.grid[idx] = true;

            // Update row and column scores
            self.update_scores(pos);
        }
    }

    fn update_scores(&mut self, pos: BoardPosition) {
        self.state.update_row_score(pos);
        self.state.update_col_score(pos);
    }

    fn is_row_filled_2(&self, y: isize) -> bool {
        (0..self.width).all(|x| self.is_cell_filled(BoardPosition { x, y }))
    }

    fn is_row_filled(&self, y: isize) -> bool {
        match self.row_score(y) {
            Some(score) => score == self.width,
            None => false,
        }
    }

    /************************ Utility functions *******************************/

    // row-ordered 2D to 1D indexing
    #[inline]
    fn idx(&self, x: isize, y: isize) -> Option<usize> {
        // Check bounds first (including negative values)
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            println!("Warning: out-of-bounds x: {}, y: {}", x, y);
            return None;
        }
        // Safe to convert to usize now
        Some((y * self.width + x) as usize)
    }

    fn de_idx(&self, index: usize) -> Option<(isize, isize)> {
        if index >= self.state.grid.len() {
            return None;
        }
        let y = index as isize / self.width;
        let x = index as isize % self.width;
        Some((x, y))
    }

    pub fn mid_x(&self) -> isize {
        // note: in Rust, this always rounds down
        self.width / 2
    }

    pub fn row_score(&self, row: isize) -> Option<isize> {
        if row >= self.height || row < 0 {
            println!("Warning: out-of-bounds y: {}", row);
            return None;
        }
        Some(self.state.row_score[row as usize])
    }

    pub fn col_score(&self, col: isize) -> Option<isize> {
        if col >= self.width || col < 0 {
            println!("Warning: out-of-bounds x: {}", col);
            return None;
        }
        Some(self.state.col_score[col as usize])
    }
}

#[derive(Debug, Clone)]
struct BoardState {
    grid: Vec<bool>,       // which cells are filled
    row_score: Vec<isize>, // how many cells are filled in each row
    col_score: Vec<isize>, // filled height of each row
}

impl BoardState {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            grid: vec![false; width * height],
            row_score: vec![0; height],
            col_score: vec![0; width],
        }
    }

    pub fn update_row_score(&mut self, pos: BoardPosition) {
        self.row_score[pos.y as usize] += 1;
    }

    pub fn update_col_score(&mut self, pos: BoardPosition) {
        if pos.y > self.col_score[pos.x as usize] {
            self.col_score[pos.x as usize] = pos.y;
        }
    }
}
