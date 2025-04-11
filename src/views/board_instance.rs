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
const CLEAR_DURATION: f32 = 0.15;
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

pub enum PlayerInput {
    L,
    R,
    HardDrop,
    Rotate,
    Pause,
}

pub struct BoardInstance {
    pub id: String,
    board: Board,   // the internal board logic
    location: Vec2, // screen location of the BoardInstance
    cell_size: f32, // size of the grid cells

    color: Rgba, // color of cells

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
        Self {
            id: id.to_owned(),
            board: Board::new(width, height),
            location,
            cell_size,

            color: rgba(0.51, 0.81, 0.94, 1.0),

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

            GameState::Clearing => {}

            GameState::GameOver => {
                // Grid has been filled to the top

                // gameover state
            }

            GameState::Paused => {
                // Pause the game
                if let Some(input) = input {
                    self.handle_input(input);
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
            x: self.board.mid_x() - piece_type.max_x(0) / 2,
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

    fn clear_lines(&mut self, rows: Vec<isize>) {
        for row in rows {
            println!("Clearing row {}", row);
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
    }

    fn draw_cell(&self, draw: &Draw, pos: BoardPosition, color: Rgba) {
        let screen_x = self.location.x + (pos.x as f32 * self.cell_size)
            - (self.board.width as f32 * self.cell_size / 2.0);
        let screen_y = self.location.y + (pos.y as f32 * self.cell_size)
            - (self.board.height as f32 * self.cell_size / 2.0);

        // Draw block
        draw.rect()
            .stroke_weight(1.0)
            .stroke(BLACK)
            .x_y(screen_x, screen_y)
            .w_h(self.cell_size, self.cell_size) // cell size
            .color(color); // color
    }

    fn draw_unfilled_cell(&self, draw: &Draw, pos: BoardPosition) {
        let screen_x = self.location.x + (pos.x as f32 * self.cell_size)
            - (self.board.width as f32 * self.cell_size / 2.0);
        let screen_y = self.location.y + (pos.y as f32 * self.cell_size)
            - (self.board.height as f32 * self.cell_size / 2.0);

        // Draw block
        draw.rect()
            .stroke_weight(1.0)
            .stroke(WHITE)
            .x_y(screen_x, screen_y)
            .w_h(self.cell_size, self.cell_size) // cell size
            .color(BLACK); // color
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
