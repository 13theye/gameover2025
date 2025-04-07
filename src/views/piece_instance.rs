// src/views/piece_instance.rs
//
//
// Defining the individual pieces on the screen

use crate::models::{Piece, PieceType};
use nannou::prelude::*;

pub struct Position {
    x: u16,
    y: u16,
}
pub enum RotationDirection {
    Cw,
    Ccw,
}

pub struct PieceInstance {
    piece: Piece,
    color: Rgba,
    rotation_index: usize,
    position: Position,
}

impl PieceInstance {
    pub fn new(typ: PieceType, color: Rgba, position: Position) -> Self {
        Self {
            piece: Piece::new(typ),
            color,
            rotation_index: 0,
            position,
        }
    }

    fn rotate(&self, direction: RotationDirection) -> &Vec<(i8, i8)> {
        let count = self.piece.rotations.len();

        let inx = match direction {
            RotationDirection::Cw => (self.rotation_index + 1) % count,
            RotationDirection::Ccw => (self.rotation_index + count - 1) % count,
        };

        &self.piece.rotations[inx]
    }
}
