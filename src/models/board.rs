// src/models/board.rs
//
// Defining the Tetris Board model

use crate::views::{BoardPosition, PieceInstance};

pub enum PlaceResult {
    PlaceOk,
    RowFilled,
    OutOfBounds,
    PlaceBad,
}

pub struct Board {
    pub width: isize,  // the overall width in cells
    pub height: isize, // the overall height in cells
    state: BoardState, // the game state
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

    pub fn drop_piece(&mut self, piece: &PieceInstance, pos: BoardPosition) -> PlaceResult {
        let (min, max) = piece.typ.minmax_x(piece.rot_idx);
        let board_min = pos.x + min;
        let board_max = pos.x + max;
        let skirt = piece.typ.skirt(piece.rot_idx);

        let mut max_height = isize::MIN;
        for x in board_min..=board_max {
            // CAUTION: assumes the piece is in a legal position
            if skirt[x as usize] > max_height {
                max_height = skirt[x as usize];
            }
        }

        self.try_place(
            piece,
            BoardPosition {
                x: pos.x,
                y: max_height,
            },
        )
    }

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

    pub fn is_cell_filled(&self, pos: BoardPosition) -> bool {
        match self.idx(pos.x, pos.y) {
            Some(inx) => self.state.grid[inx],
            None => false,
        }
    }

    fn fill_cell(&mut self, pos: BoardPosition) {
        if let Some(idx) = self.idx(pos.x, pos.y) {
            self.state.grid[idx] = true;
        }
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

    fn row_score(&self, row: isize) -> Option<isize> {
        if row >= self.height {
            println!("Warning: out-of-bounds y: {}", row);
            return None;
        }
        Some(self.state.row_score[row as usize])
    }

    fn col_score(&self, col: isize) -> Option<isize> {
        if col >= self.width {
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
}
