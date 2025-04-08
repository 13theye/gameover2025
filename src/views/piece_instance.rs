// src/views/piece_instance.rs
//
//
// Defining the individual pieces on the screen

use crate::models::PieceType;
use nannou::prelude::*;

pub struct Position {
    x: u16,
    y: u16,
}

pub enum RotationDirection {
    Cw,
    Ccw,
}

type Cells = [(isize, isize); 4];

pub struct PieceInstance {
    pub typ: PieceType,
    pub color: Rgba,
    pub rotation_idx: usize,
    pub position: Position,
}

impl PieceInstance {
    pub fn new(typ: PieceType, color: Rgba, position: Position) -> Self {
        Self {
            typ,
            color,
            rotation_idx: 0,
            position,
        }
    }

    pub fn cells(&self) -> &Cells {
        self.typ.get_rotation(self.rotation_idx)
    }

    fn rotate(&mut self, direction: RotationDirection) -> &Cells {
        let count = self.typ.rotation_count();

        let inx = match direction {
            RotationDirection::Cw => (self.rotation_idx + 1) % count,
            RotationDirection::Ccw => (self.rotation_idx + count - 1) % count,
        };

        self.rotation_idx = inx;
        self.typ.get_rotation(inx)
    }
}
