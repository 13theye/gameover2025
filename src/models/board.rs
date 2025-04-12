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
    pub width: isize,                // overall width in cells
    pub height: isize,               // overall height in cells
    state: BoardState,               // grid state
    prev_state: BoardState,          // previous grid state for testing positions
    saved_state: Option<BoardState>, // saved state for pausing
}

impl Board {
    pub fn new(width: usize, height: usize) -> Self {
        let prev_state = BoardState::new(width, height);
        Self {
            width: width as isize,
            height: height as isize,
            state: prev_state.clone(),
            prev_state,
            saved_state: None,
        }
    }

    /************************ Piece Placement *******************************/

    // Sees if the next placement is valid
    pub fn try_place(&mut self, piece: &PieceInstance, board_pos: BoardPosition) -> PlaceResult {
        // First check if the piece's position is valid

        for &(dx, dy) in piece.cells() {
            let cell_pos = BoardPosition {
                x: board_pos.x + dx,
                y: board_pos.y + dy,
            };

            println!("  Cell at {:?}", cell_pos);

            if self.idx(cell_pos.x, cell_pos.y).is_none() {
                println!("Position: {:?} is OOB (cell at {:?}", board_pos, (dx, dy));
                return PlaceResult::OutOfBounds;
            }

            if self.is_cell_filled(cell_pos) {
                println!(
                    "Position: {:?} is occupied (cell at {:?}",
                    board_pos,
                    (dx, dy)
                );
                return PlaceResult::PlaceBad;
            }
        }

        // Clone current state
        self.prev_state = self.state.clone();

        // Check if cells would be filled
        let mut test_piece = piece.clone();
        test_piece.position = board_pos;
        let row_filled = self.fills_row(&test_piece);

        // Unwind temporary changes
        std::mem::swap(&mut self.state, &mut self.prev_state);

        if row_filled {
            return PlaceResult::RowFilled;
        }

        // If we reach here, no row is filled.
        println!("Position: {:?} is OK", board_pos);
        PlaceResult::PlaceOk
    }

    fn fills_row(&mut self, piece: &PieceInstance) -> bool {
        piece.cells().iter().any(|&(dx, dy)| {
            let cell_pos = BoardPosition {
                x: piece.position.x + dx,
                y: piece.position.y + dy,
            };

            matches!(self.fill_cell(cell_pos), PlaceResult::RowFilled)
        })
    }

    // commit a pre-validated piece, returns any a Vec of any filled rows
    pub fn commit_piece(&mut self, piece: &PieceInstance) -> Option<Vec<isize>> {
        let filled_rows = piece
            .cells()
            .iter()
            .filter_map(|&(dx, dy)| {
                let cell_pos = BoardPosition {
                    x: piece.position.x + dx,
                    y: piece.position.y + dy,
                };

                // Remember the y-index of each row that has been filled
                (self.fill_cell(cell_pos) == PlaceResult::RowFilled).then_some(cell_pos.y)
            })
            .collect::<Vec<isize>>();

        (!filled_rows.is_empty()).then_some(filled_rows)
    }

    // Find the lowest legal place for piece in its current x-position
    pub fn get_drop_location(&self, piece: &PieceInstance) -> BoardPosition {
        let skirt = piece.typ.skirt(piece.rot_idx);

        // Calculate grid min/max x
        let (min_dx, max_dx) = piece.typ.minmax_x(piece.rot_idx);

        // Find the drop height
        let mut min_required_y = isize::MAX;

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
            let col_score = self.col_score(board_x).unwrap_or(0);

            // calculate the required valid y value for this column
            let required_y = if col_score == 0 {
                0 - skirt_val // place at grid bottom
            } else {
                col_score - skirt_val // place above highest piece
            };

            if required_y < min_required_y {
                min_required_y = required_y;
            }
        }

        // 0 is the minimum y value
        //let final_y: isize = std::cmp::max(0, max_required_y);
        println!("Drop location y is {}", min_required_y);

        BoardPosition {
            x: piece.position.x,
            y: min_required_y,
        }
    }

    fn fill_cell(&mut self, pos: BoardPosition) -> PlaceResult {
        self.idx(pos.x, pos.y)
            .map(|idx| {
                self.state.grid[idx] = true;
                self.state.update_col_score(pos);

                // Notice if the row has been filled while updating row score
                if self.state.update_row_score(pos) == self.width {
                    PlaceResult::RowFilled
                } else {
                    PlaceResult::PlaceOk
                }
            })
            // Invalid index means OOB
            .unwrap_or(PlaceResult::OutOfBounds)
    }

    pub fn is_cell_filled(&self, pos: BoardPosition) -> bool {
        self.idx(pos.x, pos.y)
            .map(|idx| self.state.grid[idx])
            .unwrap_or(false)
    }

    /************************ Row clearing functions ***************************/

    pub fn clear_rows(&mut self, rows: &[isize]) {
        // Sort rows in descending order
        let mut sorted_rows = rows.to_vec();
        sorted_rows.sort_by(|a, b| b.cmp(a));

        // Clear each row
        for &row in sorted_rows.iter() {
            self.clear_row(row);
        }

        // Slide rows down, starting from lowest cleared row
        if let Some(&lowest_row) = sorted_rows.last() {
            self.slide_rows_down(lowest_row, sorted_rows.len() as isize);
            self.adjust_column_heights(lowest_row);
        }
    }

    fn clear_row(&mut self, row: isize) {
        for x in 0..self.width {
            if let Some(idx) = self.idx(x, row) {
                self.state.grid[idx] = false;
            }
        }

        if row >= 0 && row < self.height {
            self.state.reset_row_score(row);
        }
    }

    fn slide_rows_down(&mut self, start_row: isize, count: isize) {
        for y in (start_row + 1)..self.height {
            for x in 0..self.width {
                let target_y = y - count;

                if target_y >= 0 {
                    let source_cell = self.is_cell_filled(BoardPosition { x, y });

                    // Update target cell
                    if let Some(idx) = self.idx(x, target_y) {
                        self.state.grid[idx] = source_cell;
                    }
                }
            }

            // Update row score
            if y < self.height && (y - count) >= 0 {
                self.state.row_score[(y - count) as usize] = self.state.row_score[y as usize];
            }
        }

        // Clear the top rows that were moved down
        for y in (self.height - count)..self.height {
            if y >= 0 {
                self.clear_row(y);
            }
        }
    }

    fn adjust_column_heights(&mut self, lowest_cleared_row: isize) {
        for x in 0..self.width as usize {
            // Only recalculate if the column had a non-zero height
            if self.state.col_score[x] > 0 {
                // Start from the previous height or the lowest cleared row, whichever is higher
                let start_y = std::cmp::max(self.state.col_score[x] - 1, lowest_cleared_row);

                // Find the new highest cell by scanning downward
                let mut new_height = 0;
                for y in (0..=start_y).rev() {
                    if self.is_cell_filled(BoardPosition { x: x as isize, y }) {
                        new_height = y + 1;
                        break;
                    }
                }

                self.state.col_score[x] = new_height;
            }
        }
    }

    /************************ Geometry functions *******************************/

    pub fn midpoint_x(&self) -> isize {
        // note: in Rust, this always rounds down
        self.width / 2
    }

    /************************ Utility functions *******************************/

    // row-ordered 2D to 1D indexing
    #[inline]
    fn idx(&self, x: isize, y: isize) -> Option<usize> {
        // Check bounds first (including negative values)
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return None;
        }
        // Safe to convert to usize now
        Some((y * self.width + x) as usize)
    }

    fn _de_idx(&self, index: usize) -> Option<(isize, isize)> {
        if index >= self.state.grid.len() {
            return None;
        }
        let y = index as isize / self.width;
        let x = index as isize % self.width;
        Some((x, y))
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

    pub fn col_score_all(&self) -> &Vec<isize> {
        &self.state.col_score
    }

    pub fn save_state(&mut self) {
        self.saved_state = Some(self.state.clone());
    }

    pub fn resume_state(&mut self) {
        if let Some(state) = &self.saved_state {
            self.state = state.clone();
        }
    }
}

#[derive(Debug, Clone)]
struct BoardState {
    grid: Vec<bool>,       // which cells are filled
    row_score: Vec<isize>, // how many cells are filled in each row
    col_score: Vec<isize>, // height of the highest UNfilled cell of each col
}

impl BoardState {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            grid: vec![false; width * height],
            row_score: vec![0; height],
            col_score: vec![0; width],
        }
    }

    pub fn reset_row_score(&mut self, row: isize) {
        self.row_score[row as usize] = 0;
    }

    pub fn update_row_score(&mut self, pos: BoardPosition) -> isize {
        let score = &mut self.row_score[pos.y as usize];
        *score += 1;
        *score
    }

    pub fn update_col_score(&mut self, pos: BoardPosition) {
        if pos.y >= self.col_score[pos.x as usize] {
            self.col_score[pos.x as usize] = pos.y + 1;
        }
    }
}
