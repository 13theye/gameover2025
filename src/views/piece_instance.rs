// src/views/piece_instance.rs
//
//
// Defining the individual pieces on the screen

use crate::models::PieceType;
use nannou::prelude::*;

pub enum RotationDirection {
    Cw,
    Ccw,
}

// Board position of a piece
#[derive(Debug, Copy, Clone)]
pub struct BoardPosition {
    pub x: isize,
    pub y: isize,
}

type Cells = [(isize, isize); 4];

pub struct PieceInstance {
    pub typ: PieceType,
    pub color: Rgba,
    pub rot_idx: usize, // rotation index
    pub position: BoardPosition,
}

impl PieceInstance {
    pub fn new(typ: PieceType, color: Rgba, position: BoardPosition) -> Self {
        Self {
            typ,
            color,
            rot_idx: 0,
            position,
        }
    }

    pub fn cells(&self) -> &Cells {
        self.typ.get_rotation(self.rot_idx)
    }

    fn rotate(&mut self, direction: RotationDirection) -> &Cells {
        let count = self.typ.rotation_count();

        let inx = match direction {
            RotationDirection::Cw => (self.rot_idx + 1) % count,
            RotationDirection::Ccw => (self.rot_idx + count - 1) % count,
        };

        self.rot_idx = inx;
        self.typ.get_rotation(inx)
    }

    /***************** Coordinate Translation Methods ******************** */

    // Get the absolute board positions for all cells
    pub fn board_positions(&self) -> Vec<BoardPosition> {
        self.cells()
            .iter()
            .map(|&(dx, dy)| BoardPosition {
                x: self.position.x + dx,
                y: self.position.y + dy,
            })
            .collect()
    }

    // Get the board position for a specific cell
    pub fn board_cell_position(&self, cell_idx: usize) -> BoardPosition {
        let (dx, dy) = self.cells()[cell_idx];
        BoardPosition {
            x: self.position.x + dx,
            y: self.position.y + dy,
        }
    }

    // Get a hypothetical position if this piece were at a different position
    pub fn board_test_positions(&self, test_pos: BoardPosition) -> Vec<BoardPosition> {
        self.cells()
            .iter()
            .map(|&(dx, dy)| BoardPosition {
                x: test_pos.x + dx,
                y: test_pos.y + dy,
            })
            .collect()
    }
}
