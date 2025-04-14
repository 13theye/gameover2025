// src/models/board.rs
//
// Defining the Tetris Board model

use crate::views::{BoardPosition, PieceInstance};

const DEBUG: bool = false;

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
    backup_state: BoardState,        // previous grid state for testing positions
    saved_state: Option<BoardState>, // saved state for pausing
}

impl Board {
    pub fn new(width: usize, height: usize) -> Self {
        let prev_state = BoardState::new(width, height);
        Self {
            width: width as isize,
            height: height as isize,
            state: prev_state.clone(),
            backup_state: prev_state,
            saved_state: None,
        }
    }

    /************************ Piece Placement *******************************/

    // Check validity of desired piece placement, returns result of placement
    pub fn try_place(&mut self, piece: &PieceInstance, board_pos: BoardPosition) -> PlaceResult {
        // First check if the piece's position is valid

        for &(dx, dy) in piece.cells() {
            let cell_pos = BoardPosition {
                x: board_pos.x + dx,
                y: board_pos.y + dy,
            };

            if self.idx(cell_pos.x, cell_pos.y).is_none() {
                if DEBUG {
                    println!(
                        "Try Position: {:?} is OOB -- cell at {:?}",
                        board_pos,
                        (dx, dy)
                    );
                }
                return PlaceResult::OutOfBounds;
            }

            if self.is_cell_filled(cell_pos) {
                if DEBUG {
                    println!(
                        "Try Position: {:?} is occupied -- cell at {:?}",
                        board_pos,
                        (dx, dy)
                    );
                }
                return PlaceResult::PlaceBad;
            }
        }

        // Clone current state
        self.backup_state = self.state.clone();

        // Check if cells would be filled
        let mut test_piece = piece.clone();
        test_piece.position = board_pos;
        let row_filled = self.fills_row(&test_piece);

        // Unwind temporary changes
        std::mem::swap(&mut self.state, &mut self.backup_state);

        if row_filled {
            return PlaceResult::RowFilled;
        }

        // If we reach here, no row is filled.
        if DEBUG {
            println!("Try Position: {:?} is OK", board_pos);
        }
        PlaceResult::PlaceOk
    }

    // Quick check that a piece would fill a row
    fn fills_row(&mut self, piece: &PieceInstance) -> bool {
        piece.cells().iter().any(|&(dx, dy)| {
            let cell_pos = BoardPosition {
                x: piece.position.x + dx,
                y: piece.position.y + dy,
            };

            matches!(self.fill_cell(cell_pos), PlaceResult::RowFilled)
        })
    }

    // commit all cells of a pre-validated piece, returns any a Vec of any filled rows
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

    // Fill the cell in the Grid abstraction & update the col/row scores
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

    /************************ Piece Drop *******************************/

    // Find the lowest legal place for piece in its current x-position
    // This is the normal route and uses a quick calculation using col_score
    pub fn calculate_drop(&mut self, piece: &PieceInstance) -> (BoardPosition, PlaceResult) {
        // Use brute force method if piece is below overhang (col_score not useful)
        if self.is_below_overhang(piece) {
            return self.slow_calculate_drop(piece);
        }

        let skirt = piece.typ.skirt(piece.rot_idx);

        // Calculate grid min/max x
        let (min_dx, max_dx) = piece.typ.minmax_x(piece.rot_idx);

        // Find the max drop height valid for all cells of the piece:
        let mut max_required_y = isize::MIN;

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

            if required_y > max_required_y {
                max_required_y = required_y;
            }
        }

        let drop_position = BoardPosition {
            x: piece.position.x,
            y: max_required_y,
        };

        if DEBUG {
            println!("  Sending to verification: {:?}", drop_position);
        }

        // check that position is valid
        self.verify_drop_location(piece, drop_position)
    }

    // For pieces below an overhang, col_score won't work, so step through each
    // cell position and check for the drop height.
    fn slow_calculate_drop(&mut self, piece: &PieceInstance) -> (BoardPosition, PlaceResult) {
        if DEBUG {
            println!("Piece below overhang, starting brute force drop calculation.")
        }

        let mut test_pos = piece.position;
        let mut last_valid_pos = test_pos;
        let mut last_result = PlaceResult::PlaceOk; // current position should be Ok

        // Move down once cell at a time until collision
        loop {
            let next_pos = BoardPosition {
                x: test_pos.x,
                y: test_pos.y - 1,
            };

            let result = self.try_place(piece, next_pos);
            match result {
                PlaceResult::PlaceOk | PlaceResult::RowFilled => {
                    // Try next row below this
                    last_valid_pos = next_pos;
                    last_result = result;
                    test_pos = next_pos;
                }
                _ => {
                    // Can't move down further
                    break;
                }
            }
        }

        (last_valid_pos, last_result)
    }

    // Take a drop location and test for collisions. If collision, move up 1 row
    // and try again until no collisions remain.
    fn verify_drop_location(
        &mut self,
        piece: &PieceInstance,
        mut pos: BoardPosition,
    ) -> (BoardPosition, PlaceResult) {
        loop {
            if DEBUG {
                println!("  Verification: {:?}", pos);
            }
            let verification = self.try_place(piece, pos);
            match verification {
                PlaceResult::PlaceOk | PlaceResult::RowFilled => {
                    return (pos, verification);
                } // position is good
                PlaceResult::OutOfBounds => {
                    if DEBUG {
                        println!("   Verification OOB: {:?}", pos);
                    }
                    pos.y += 1;
                } // let this be caught downstream
                PlaceResult::PlaceBad => {
                    pos.y += 1;
                }
            }
        }
    }

    fn is_below_overhang(&self, piece: &PieceInstance) -> bool {
        piece.cells().iter().any(|&(dx, dy)| {
            let cell_pos = BoardPosition {
                x: piece.position.x + dx,
                y: piece.position.y + dy,
            };

            // Check if this cell is below an overhang
            if let Some(score) = self.col_score(cell_pos.x) {
                return cell_pos.y < (score - 1);
            }

            false // Not below an overhang if column score is unavailable
        })
    }

    /************************ Row clearing functions ***************************/

    // Orchestrate row clearing and sliding on RowFilled
    pub fn clear_rows(&mut self, rows: &[isize]) {
        // Sort rows in descending order
        let mut sorted_rows = rows.to_vec();
        sorted_rows.sort_by(|a, b| b.cmp(a));

        if DEBUG {
            println!("Rows to clear: {:?}", sorted_rows)
        }

        // Move rows by an amount depending on how many rows were cleared below them
        self.handle_sliding(&sorted_rows);

        // Adjust column heights
        if let Some(&lowest_row) = sorted_rows.last() {
            self.adjust_col_scores(lowest_row);
        }
    }

    // Handle row sliding based on a row's position
    fn handle_sliding(&mut self, cleared_rows: &[isize]) {
        if cleared_rows.is_empty() {
            return;
        }

        // We'll need these numbers to determine the scope of row clearing.
        let highest_filled_row = *self.col_score_all().iter().max().unwrap_or(&self.height);
        let min_cleared = *cleared_rows.iter().min().unwrap_or(&0);
        let max_cleared = *cleared_rows.iter().max().unwrap_or(&self.height);
        let count = cleared_rows.len() as isize;

        if DEBUG {
            println!("Sliding rows...");
            println!(
                "min: {}., max: {}, count: {}",
                min_cleared, max_cleared, count
            );
        }

        // First, handle any uncleared rows within the cleared range
        for row in min_cleared..max_cleared {
            // Skip the rows that were cleared -- we're looking for uncleared rows.
            if cleared_rows.contains(&row) {
                continue;
            }

            let slide_val: isize = cleared_rows.iter().filter(|&&y| row > y).count() as isize;
            if DEBUG {
                println!(
                    "Found uncleared row between. Row: {}, Slide_val: {}",
                    row, slide_val
                )
            }

            self.slide_row_down(row, slide_val);
        }

        if DEBUG {
            println!("Highest filled row: {}", highest_filled_row);
        }

        // Next, handle rows above the cleared range
        for row in (max_cleared + 1)..highest_filled_row {
            if DEBUG {
                println!("Sliding row: {}, Slide_val: {}", row, count);
            }

            // Move all the cells down
            self.slide_row_down(row, count);

            // Now that the cells have been moved, clear the row.
            self.clear_row(row)
        }
    }

    // Slide down an individual row, copying grid and row score
    fn slide_row_down(&mut self, row: isize, slide_val: isize) {
        let target_y = row - slide_val;

        // Clear the target row
        self.clear_row(target_y);

        // Move each cell to the target row
        for x in 0..self.width {
            let source_cell = self.is_cell_filled(BoardPosition { x, y: row });
            if let Some(idx) = self.idx(x, target_y) {
                self.state.grid[idx] = source_cell;
            }
        }

        // Update row score by copying the old score to the new row
        if row < self.height && target_y >= 0 {
            self.state.row_score[target_y as usize] = self.state.row_score[row as usize];
        }
    }

    // Clear a row completely
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

    // Recalculate col_score after sliding/clearing operations
    fn adjust_col_scores(&mut self, lowest_cleared_row: isize) {
        for x in 0..self.width {
            // Only recalculate if the column had a non-zero height
            let x_idx = x as usize;
            if self.state.col_score[x_idx] > 0 {
                // Start from the previous height or the lowest cleared row, whichever is higher
                let start_y = std::cmp::max(self.state.col_score[x_idx] - 1, lowest_cleared_row);

                // Find the new highest cell by scanning downward
                let mut new_height = 0;
                for y in (0..=start_y).rev() {
                    if self.is_cell_filled(BoardPosition { x, y }) {
                        new_height = y + 1;
                        break;
                    }
                }

                self.state.col_score[x_idx] = new_height;
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
