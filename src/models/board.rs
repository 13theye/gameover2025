// src/models/board.rs
//
// Defining the Tetris Board model

use crate::views::PieceInstance;

pub enum PlaceResult {
    PlaceOk,
    RowFilled,
    OutOfBounds,
    PlaceBad,
}

pub struct Board {
    pub width: isize,
    pub height: isize,
    grid: Vec<bool>,
}

impl Board {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width: width as isize,
            height: height as isize,
            grid: vec![false; width * height],
        }
    }

    /************************ Piece Placement *******************************/

    // Sees if the next placement is valid
    pub fn try_place(&mut self, piece: &PieceInstance, pos_x: isize, pos_y: isize) -> PlaceResult {
        for &(dx, dy) in piece.cells() {
            let x = pos_x + dx;
            let y = pos_y + dy;

            if self.idx(x, y).is_none() {
                return PlaceResult::OutOfBounds;
            }

            if self.is_cell_filled(x, y) {
                return PlaceResult::PlaceBad;
            }
        }

        // Place the piece
        for &(dx, dy) in piece.cells() {
            self.fill_cell(pos_x + dx, pos_y + dy)
        }

        PlaceResult::PlaceOk
    }

    pub fn is_cell_filled(&self, x: isize, y: isize) -> bool {
        match self.idx(x, y) {
            Some(inx) => self.grid[inx],
            None => false,
        }
    }

    fn fill_cell(&mut self, x: isize, y: isize) {
        if let Some(idx) = self.idx(x, y) {
            self.grid[idx] = true;
        }
    }

    fn is_row_filled(&self, y: isize) -> bool {
        (0..self.width).all(|x| self.is_cell_filled(x, y))
    }

    /************************ Utility functions *******************************/
    #[inline]
    fn idx(&self, x: isize, y: isize) -> Option<usize> {
        // Check bounds first (including negative values)
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            println!("Warning: out-of-bounds x:{}, y:{}", x, y);
            return None;
        }
        // Safe to convert to usize now
        Some((y * self.width + x) as usize)
    }

    #[inline]
    fn de_idx(&self, index: usize) -> Option<(isize, isize)> {
        if index >= self.grid.len() {
            return None;
        }
        let y = index as isize / self.width;
        let x = index as isize % self.width;
        Some((x, y))
    }
    fn mid_x(&self) -> isize {
        // note: in Rust, this always rounds down
        self.width / 2
    }
}
