// src/views/board_instance.rs
//
// An individual Tetris board

use crate::{
    models::{Board, PieceType},
    views::PieceInstance,
};
use nannou::prelude::*;

pub struct BoardInstance {
    pub id: String,
    board: Board,
    location: Vec2,

    cell_size: f32,
}

impl BoardInstance {
    pub fn new(id: &str, location: Vec2, width: usize, height: usize, cell_size: f32) -> Self {
        Self {
            id: id.to_owned(),
            board: Board::new(width, height),
            location,
            cell_size,
        }
    }

    /************************ Drawing functions *******************************/

    pub fn draw(&self, draw: &Draw) {
        for y in 0..self.board.height {
            for x in 0..self.board.width {
                if self.board.is_cell_filled(x, y) {
                    // convert to Nannou coords
                    let screen_x = self.location.x + (x as f32 * self.cell_size)
                        - (self.board.width as f32 * self.cell_size / 2.0);
                    let screen_y = self.location.x + (y as f32 * self.cell_size)
                        - (self.board.height as f32 * self.cell_size / 2.0);

                    // Draw block
                    draw.rect()
                        .stroke_weight(2.0)
                        .stroke(WHITE)
                        .x_y(screen_x, screen_y)
                        .w_h(self.cell_size, self.cell_size) // cell size
                        .color(RED); // color
                }
            }
        }
    }

    /************************ Utility functions *******************************/

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn board_mut(&mut self) -> &mut Board {
        &mut self.board
    }
}
