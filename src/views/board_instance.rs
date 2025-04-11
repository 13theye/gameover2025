// src/views/board_instance.rs
//
// An individual Tetris board

use crate::{
    models::{Board, PieceType, PlaceResult},
    utils::Timer,
    views::{BoardPosition, PieceInstance, RotationDirection},
};
use nannou::{
    prelude::*,
    rand::{rngs::ThreadRng, Rng},
};

// helps visualize grid for debugging
const DEBUG: bool = true;

// hard-coded animation timers
const CLEAR_DURATION: f32 = 0.5;
const SLIDE_DURATION: f32 = 0.15;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum GameState {
    Ready,    // ready to spawn a new piece
    Falling,  // Piece is falling
    Locking,  // Piece has landed and is about to commit
    Clearing, // Clearing the completed rows
    GameOver, // Game over
    Paused,
}

#[derive(PartialEq)]
pub enum PlayerInput {
    L,
    R,
    HardDrop,
    Rotate,
    Pause,
    SaveState,
    ResumeState,
}

pub struct BoardInstance {
    pub id: String,
    pub board: Board,   // the internal board logic
    pub location: Vec2, // screen location of the BoardInstance
    pub cell_size: f32, // size of the grid cells

    color: Rgba,          // color of cells
    boundary_color: Rgba, // color of outer boundary

    game_state: GameState,              // state of the game loops
    prev_game_state: Option<GameState>, // used to come back from pause, for example
    timers: GameTimers,                 // timers used in the game

    rows_to_clear: Option<Vec<isize>>, // rows idxs for the Clearing state to clear
    active_piece: Option<PieceInstance>, // the currently active piece
}

impl BoardInstance {
    pub fn new(
        id: &str,
        location: Vec2,
        width: usize,
        height: usize,
        cell_size: f32,
        gravity_interval: f32,
        lock_delay: f32,
    ) -> Self {
        let boundary_color = rgba(0.22, 0.902, 0.082, 1.0);
        let piece_color = rgba(0.235, 0.851, 0.11, 1.0);

        Self {
            id: id.to_owned(),
            board: Board::new(width, height),
            location,
            cell_size,

            color: piece_color,
            boundary_color,

            game_state: GameState::Ready,
            prev_game_state: None,
            timers: GameTimers::new(gravity_interval, lock_delay, CLEAR_DURATION, SLIDE_DURATION),

            rows_to_clear: None,
            active_piece: None,
        }
    }

    /************************ Update orchestrator *******************************/

    // Game State Machine
    pub fn update(&mut self, dt: f32, input: &Option<PlayerInput>, rng: &mut ThreadRng) {
        match self.game_state {
            GameState::Ready => {
                // Spawn a new piece
                if self.spawn_new_piece(rng) {
                    self.game_state = GameState::Falling;
                } else {
                    self.game_state = GameState::GameOver;
                }
            }

            GameState::Falling => {
                // Handle an active piece

                if let Some(input) = input {
                    self.handle_input(input);
                }

                if self.timers.gravity.tick(dt) && !self.apply_gravity() {
                    // Drop the piece 1 cell per gravity_interval
                    self.game_state = GameState::Locking;
                }
            }

            GameState::Locking => {
                // Last-minute adjustment period for piece

                if let Some(input) = input {
                    self.handle_input(input);
                }

                if let Some(piece) = self.active_piece.as_mut() {
                    // Check if the piece can now fall
                    // because of some input during the Locking period

                    let next_pos = BoardPosition {
                        x: piece.position.x,
                        y: piece.position.y - 1,
                    };

                    if self.board.try_place(piece, next_pos) == PlaceResult::PlaceOk {
                        piece.position = next_pos;
                        // Reset timers when piece moves
                        self.timers.lock.reset();
                        self.timers.gravity.reset();
                        self.game_state = GameState::Falling;
                    }
                }

                if self.timers.lock.tick(dt) {
                    // Lock the piece, commit, check for lines, return to Ready state.

                    self.rows_to_clear = self.commit_piece();
                    if self.rows_to_clear.is_some() {
                        self.game_state = GameState::Clearing;
                    } else {
                        self.game_state = GameState::Ready;
                    }

                    if DEBUG {
                        print_col_score(self.board.col_score_all());
                    }
                }
            }

            GameState::Clearing => {
                if let Some(input) = input {
                    self.handle_input(input);
                }

                if self.timers.clear_animation.tick(dt) {
                    // Animation done, now update the model
                    if let Some(rows) = self.rows_to_clear.take() {
                        self.clear_rows(&rows)
                    }

                    // Reset timer and return to Ready state
                    self.timers.clear_animation.reset();
                    self.game_state = GameState::Ready;
                }
            }

            GameState::GameOver => {
                // Grid has been filled to the top

                // gameover state
            }

            GameState::Paused => {
                // Pause the game
                if let Some(input) = input {
                    self.handle_pause_input(input);
                }
            }
        }
    }

    /************************ Update loop methods ***************************/
    fn spawn_new_piece(&mut self, rng: &mut ThreadRng) -> bool {
        // Randomize new piece properties and create
        let piece_type = self.get_random_piece_type(rng);
        let color = self.get_piece_color();

        let spawn_pos = BoardPosition {
            x: self.board.midpoint_x() - piece_type.max_x(0) / 2,
            y: self.board.height - piece_type.max_y(0) - 1,
        };

        let new_piece = PieceInstance::new(piece_type, color, spawn_pos);

        // Verify that piece can be placed
        let can_place = matches!(
            self.board.try_place(&new_piece, spawn_pos),
            PlaceResult::PlaceOk | PlaceResult::RowFilled
        );

        if can_place {
            if DEBUG {
                spawn_new_piece_msg(&new_piece);
            }

            self.active_piece = Some(new_piece);
        }

        can_place
    }

    // Freeze a piece in place
    fn commit_piece(&mut self) -> Option<Vec<isize>> {
        self.active_piece
            .take()
            .and_then(|piece| self.board.commit_piece(&piece))
    }

    fn clear_rows(&mut self, rows: &[isize]) {
        self.board.clear_rows(rows);
        if DEBUG {
            print_col_score(self.board.col_score_all());
        }
    }

    /************************ Piece movement methods ************************/

    // Drop a piece down the board
    fn apply_gravity(&mut self) -> bool {
        let Some(piece) = self.active_piece.as_mut() else {
            return false;
        };

        let next_pos = BoardPosition {
            x: piece.position.x,
            y: piece.position.y - 1,
        };

        let can_place = matches!(
            self.board.try_place(piece, next_pos),
            PlaceResult::PlaceOk | PlaceResult::RowFilled
        );

        if can_place {
            piece.position = next_pos;
        }

        can_place
    }

    // Player-induced drop down to lowest legal position
    fn hard_drop(&mut self) {
        if let Some(piece) = self.active_piece.as_mut() {
            let drop_pos = self.board.get_drop_location(piece);

            // move piece to calculated position
            if self.move_active_piece(drop_pos) {
                // Transition to locking
                self.game_state = GameState::Locking;
                if DEBUG {
                    println!("Hard drop executed: piece at y: {}", drop_pos.y);
                }
            } else {
                println!("Hard drop failed: attempted at y: {}", drop_pos.y);
            }
        }
    }

    // Player-induced movement of piece
    fn move_active_piece(&mut self, new_pos: BoardPosition) -> bool {
        let Some(piece) = self.active_piece.as_mut() else {
            return false;
        };

        let can_place = matches!(
            self.board.try_place(piece, new_pos),
            PlaceResult::PlaceOk | PlaceResult::RowFilled
        );

        if can_place {
            piece.position = new_pos;
        }

        can_place
    }

    // Player-induced piece rotation
    // Only moves in Cw direction for now
    fn rotate_active_piece(&mut self) {
        if let Some(piece) = &mut self.active_piece {
            // Save the current rotation index
            let old_rot_idx = piece.rot_idx;

            // Perform the rotation
            piece.rotate(RotationDirection::Cw);

            // Check if the new position is valid
            if self.board.try_place(piece, piece.position) == PlaceResult::PlaceOk {
                // Rotation successful, no further action needed
            } else {
                // Revert to the previous rotation
                piece.rot_idx = old_rot_idx;
            }
        }
    }

    /************************ Piece creation methods ************************/
    fn get_random_piece_type(&self, rng: &mut ThreadRng) -> PieceType {
        let idx = rng.gen_range(0.0..7.0).trunc() as usize;
        PieceType::from_idx(idx)
    }

    fn get_piece_color(&self) -> Rgba {
        self.color
    }

    /************************ Input handling methods *******************************/

    fn handle_input(&mut self, input: &PlayerInput) {
        match input {
            PlayerInput::L => {
                if let Some(piece) = self.active_piece.as_mut() {
                    let new_pos = BoardPosition {
                        x: piece.position.x - 1,
                        y: piece.position.y,
                    };

                    self.move_active_piece(new_pos);
                }
            }
            PlayerInput::R => {
                if let Some(piece) = self.active_piece.as_mut() {
                    let new_pos = BoardPosition {
                        x: piece.position.x + 1,
                        y: piece.position.y,
                    };

                    self.move_active_piece(new_pos);
                }
            }
            PlayerInput::Rotate => {
                self.rotate_active_piece();
            }
            PlayerInput::HardDrop => {
                self.hard_drop();
            }
            PlayerInput::Pause => {
                self.handle_pause();
            }
            _ => {}
        }
    }

    fn handle_pause_input(&mut self, input: &PlayerInput) {
        // ignore everything except Pause
        match input {
            PlayerInput::Pause => {
                self.handle_pause();
            }
            PlayerInput::SaveState => {
                self.board.save_state();
                self.active_piece = None;
                self.game_state = GameState::Ready
            }
            PlayerInput::ResumeState => {
                self.board.resume_state();
                self.active_piece = None;
                self.game_state = GameState::Ready
            }
            _ => {}
        }
    }

    fn handle_pause(&mut self) {
        if self.game_state == GameState::Paused {
            // Exiting pause state
            self.game_state = self.prev_game_state.take().unwrap_or(GameState::Ready);
            self.timers.resume_all();
            // Restore timers if pause state exists
        } else {
            // Entering pause state
            self.prev_game_state = Some(self.game_state);
            self.game_state = GameState::Paused;
            self.timers.pause_all();
        }
    }

    /************************ Drawing methods *******************************/

    pub fn draw(&self, draw: &Draw) {
        // Draw the board
        for y in 0..self.board.height {
            for x in 0..self.board.width {
                let pos = BoardPosition { x, y };
                if self.board.is_cell_filled(pos) {
                    self.draw_cell(draw, pos, self.color);
                } else if DEBUG {
                    self.draw_unfilled_cell(draw, pos)
                }
            }
        }

        // Draw the active piece
        if let Some(piece) = &self.active_piece {
            for &(dx, dy) in piece.cells() {
                let pos = BoardPosition {
                    x: piece.position.x + dx,
                    y: piece.position.y + dy,
                };

                if pos.x >= 0 && pos.x < self.board.width && pos.y >= 0 && pos.y < self.board.height
                {
                    self.draw_cell(draw, pos, piece.color);
                }
            }
        }

        let effective_state = if self.game_state == GameState::Paused {
            self.prev_game_state.unwrap_or(self.game_state)
        } else {
            self.game_state
        };

        // Draw the clearing animation if effective state is Clearing state
        if effective_state == GameState::Clearing {
            self.draw_clear_animation(draw);
        }

        // Draw boundary around the board
        self.draw_boundary(draw);
    }

    fn draw_cell(&self, draw: &Draw, pos: BoardPosition, color: Rgba) {
        // Draw block
        draw.rect()
            .xy(pos.to_screen(self))
            .w_h(self.cell_size, self.cell_size) // cell size
            .color(color) // color
            .stroke_weight(1.5)
            .stroke(BLACK);
    }

    fn draw_unfilled_cell(&self, draw: &Draw, pos: BoardPosition) {
        // Draw block
        draw.rect()
            .xy(pos.to_screen(self))
            .w_h(self.cell_size, self.cell_size) // cell size
            .color(BLACK) // color
            .stroke_weight(1.5)
            .stroke(WHITE);
    }

    fn draw_clear_animation(&self, draw: &Draw) {
        let Some(rows) = &self.rows_to_clear else {
            return;
        };

        let progress = self.timers.clear_animation.progress();

        // Find row bounds
        let top_row = *rows.iter().max().unwrap_or(&0);
        let bottom_row = *rows.iter().min().unwrap_or(&0);

        // Calculate clear area
        let top_bound = BoardPosition { x: 0, y: top_row }.to_screen(self).y;
        let bottom_bound = BoardPosition {
            x: 0,
            y: bottom_row,
        }
        .to_screen(self)
        .y;

        let board_left_edge = self.location.x - (self.board.width as f32 * self.cell_size / 2.0);
        let board_width = self.board.width as f32 * self.cell_size;

        // Calculate separation based on progress. Minimum is half a cell height.
        let center_y = bottom_bound + (top_bound - bottom_bound) / 2.0;
        let separation = if top_row == bottom_row {
            progress * self.cell_size / 2.0
        } else {
            progress * (top_bound - bottom_bound)
        };

        // Line positions
        let top_y = center_y + separation;
        let bottom_y = center_y - separation;

        // Clear the area between the lines as they separate
        if progress > 0.1 {
            // Start clearing after a little bit of separation
            let clear_height = (top_y - bottom_y).abs();
            draw.rect()
                .x_y(self.location.x, center_y)
                .w_h(board_width, clear_height)
                .color(rgba(1.0, 1.0, 1.0, 0.5));
        }

        // Draw top and bottom lines
        for y_pos in [top_y, bottom_y] {
            // Main line
            draw.line()
                .points(
                    vec2(board_left_edge, y_pos),
                    vec2(board_left_edge + board_width, y_pos),
                )
                .color(rgba(1.0, 1.0, 1.0, 0.5))
                .stroke_weight(1.0);
        }
    }

    fn draw_boundary(&self, draw: &Draw) {
        draw.rect()
            .x_y(self.location.x, self.location.y)
            .w_h(
                self.board.width as f32 * self.cell_size,
                self.board.height as f32 * self.cell_size,
            )
            .stroke_weight(1.0)
            .stroke_color(self.boundary_color)
            .color(rgba(0.0, 0.0, 0.0, 0.0));
    }

    /************************ Utility methods *******************************/

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn board_mut(&mut self) -> &mut Board {
        &mut self.board
    }
}

/************************ Stdout functions *******************************/

fn spawn_new_piece_msg(piece: &PieceInstance) {
    println!("\n-- Spawned new piece --");
    println!(
        "PieceType: {:?}\nPosition:{:?}\n",
        piece.typ, piece.position
    )
}

fn print_col_score(col_score: &Vec<isize>) {
    println!("\nCol heights:");
    println!("{:?}", col_score);
}

struct GameTimers {
    gravity: Timer,
    lock: Timer,
    clear_animation: Timer,
    slide_animation: Timer,
}

impl GameTimers {
    pub fn new(
        gravity_interval: f32,
        lock_delay: f32,
        clear_duration: f32,
        slide_duration: f32,
    ) -> Self {
        Self {
            gravity: Timer::new(gravity_interval),
            lock: Timer::new(lock_delay),
            clear_animation: Timer::new(clear_duration),
            slide_animation: Timer::new(slide_duration),
        }
    }

    pub fn pause_all(&mut self) {
        self.gravity.pause();
        self.lock.pause();
        self.clear_animation.pause();
        self.slide_animation.pause();
    }

    pub fn resume_all(&mut self) {
        self.gravity.resume();
        self.lock.resume();
        self.clear_animation.resume();
        self.slide_animation.resume();
    }
}
