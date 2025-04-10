// src/views/board_instance.rs
//
// An individual Tetris board

use crate::{
    models::{Board, PieceType, PlaceResult},
    views::{BoardPosition, PieceInstance, RotationDirection},
};
use nannou::{
    prelude::*,
    rand::{rngs::ThreadRng, Rng},
};

// helps visualize grid for debugging
const DEBUG: bool = true;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum GameState {
    Ready,    // ready to spawn a new piece
    Falling,  // Piece is falling
    Locking,  // Piece has landed and is about to commit
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

struct PauseState {
    gravity_timer: f32,
    lock_timer: f32,
}

pub struct BoardInstance {
    pub id: String,
    board: Board,   // the internal board logic
    location: Vec2, // screen location of the BoardInstance
    cell_size: f32, // size of the grid cells

    color: Rgba,

    game_state: GameState,
    prev_game_state: Option<GameState>, // used to come back from pause, for example
    pause_state: Option<PauseState>,

    active_piece: Option<PieceInstance>,
    gravity_interval: f32, // time between gravity steps
    gravity_timer: f32,
    lock_delay: f32, // time before piece locks into place
    lock_timer: f32,
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
            pause_state: None,

            active_piece: None,
            gravity_interval,
            gravity_timer: 0.0,
            lock_delay,
            lock_timer: 0.0,
        }
    }

    /************************ Update orchestrator *******************************/

    pub fn update(&mut self, dt: f32, input: &Option<PlayerInput>, rng: &mut ThreadRng) {
        match self.game_state {
            // Main Game State Machine
            GameState::Ready => {
                self.spawn_new_piece(rng);
                self.game_state = GameState::Falling;
            }

            GameState::Falling => {
                if let Some(input) = input {
                    self.handle_input(input);
                }

                self.gravity_timer += dt;
                if self.gravity_timer >= self.gravity_interval {
                    self.gravity_timer = 0.0;
                    if !self.apply_gravity() {
                        self.game_state = GameState::Locking;
                    }
                }
            }

            GameState::Locking => {
                if let Some(input) = input {
                    self.handle_input(input);
                }

                // check if the piece can now fall because of some input
                // during the Locking period
                if let Some(piece) = self.active_piece.as_mut() {
                    let next_pos = BoardPosition {
                        x: piece.position.x,
                        y: piece.position.y - 1,
                    };

                    if self.board.try_place(piece, next_pos) == PlaceResult::PlaceOk {
                        piece.position = next_pos;
                        // Reset timers when piece moves
                        self.lock_timer = 0.0;
                        self.gravity_timer = 0.0;
                        self.game_state = GameState::Falling;
                    }
                }

                self.lock_timer += dt;
                if self.lock_timer >= self.lock_delay {
                    self.lock_timer = 0.0;
                    self.commit_piece();
                    print_col_score(self.board.col_score_all());
                    self.clear_lines();
                    self.game_state = GameState::Ready;
                }
            }

            GameState::GameOver => {
                // gameover state
            }

            GameState::Paused => {
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

        // Check if piece can be placed
        let can_place = self.board.try_place(&new_piece, spawn_pos);

        match can_place {
            PlaceResult::PlaceOk | PlaceResult::RowFilled => {
                if DEBUG {
                    spawn_new_piece_msg(&new_piece);
                }
                self.active_piece = Some(new_piece);
                true
            }

            PlaceResult::OutOfBounds | PlaceResult::PlaceBad => {
                self.game_state = GameState::GameOver;
                false
            }
        }
    }

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

    fn commit_piece(&mut self) {
        if let Some(piece) = &self.active_piece {
            self.board.commit_piece(piece);
            self.active_piece = None;
        }
    }

    fn clear_lines(&mut self) {}

    /************************ Piece movement methods ************************/

    fn apply_gravity(&mut self) -> bool {
        if let Some(piece) = self.active_piece.as_mut() {
            let next_pos = BoardPosition {
                x: piece.position.x,
                y: piece.position.y - 1,
            };

            let can_place = self.board.try_place(piece, next_pos);

            match can_place {
                PlaceResult::PlaceOk | PlaceResult::RowFilled => {
                    piece.position = next_pos;
                    true
                }

                PlaceResult::OutOfBounds | PlaceResult::PlaceBad => false,
            }
        } else {
            false
        }
    }

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

    fn move_active_piece(&mut self, new_pos: BoardPosition) -> bool {
        if let Some(piece) = self.active_piece.as_mut() {
            let can_place = self.board.try_place(piece, new_pos);

            match can_place {
                PlaceResult::PlaceOk | PlaceResult::RowFilled => {
                    piece.position = new_pos;
                    true
                }

                PlaceResult::OutOfBounds | PlaceResult::PlaceBad => false,
            }
        } else {
            false
        }
    }

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

    /************************ Meta methods *******************************/
    fn handle_pause(&mut self) {
        if self.game_state == GameState::Paused {
            // Exiting pause state
            if let Some(prev_game_state) = self.prev_game_state {
                // Restore previous game state
                self.game_state = prev_game_state;
                // Restore timers
                if let Some(pause_state) = &self.pause_state {
                    self.gravity_timer = pause_state.gravity_timer;
                    self.lock_timer = pause_state.lock_timer;
                }
                self.pause_state = None;
            } else {
                // Fallback if somehow we don't have a previous state
                self.game_state = GameState::Ready;
            }
        } else {
            // Entering pause state
            self.prev_game_state = Some(self.game_state);
            self.pause_state = Some(PauseState {
                gravity_timer: self.gravity_timer,
                lock_timer: self.lock_timer,
            });
            self.game_state = GameState::Paused;
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
            .stroke(WHITE)
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
