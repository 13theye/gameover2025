// src/views/mod.rs

pub mod background;
pub mod board_instance;
pub mod piece_instance;

pub use background::BackgroundManager;
pub use board_instance::{BoardInstance, PlayerInput};
pub use piece_instance::{BoardPosition, PieceInstance, RotationDirection};
