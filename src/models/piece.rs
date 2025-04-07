// src/models/piece.rs
//
//
// Defining the Tetris piece model
//
// A Piece's (0,0) is the central pivot point of rotation

#[derive(Debug, PartialEq)]
pub enum PieceType {
    I,
    J,
    L,
    S,
    Z,
    T,
    O,
}

#[derive(Debug)]
pub struct Piece {
    pub typ: PieceType,
    pub rotations: Vec<Vec<(i8, i8)>>,
}

impl Piece {
    pub fn new(typ: PieceType) -> Self {
        let rotations: Vec<Vec<(i8, i8)>> = match typ {
            // (0,0) is the central pivot point
            PieceType::I => vec![
                vec![(-1, 0), (0, 0), (1, 0), (2, 0)], // 0°
                vec![(0, -1), (0, 0), (0, 1), (0, 2)], // 90°
                vec![(-1, 0), (0, 0), (1, 0), (2, 0)], // 180°
                vec![(0, -1), (0, 0), (0, 1), (0, 2)], // 270°
            ],
            PieceType::J => vec![
                vec![(-1, 0), (0, 0), (1, 0), (-1, 1)],  // 0°
                vec![(0, -1), (0, 0), (0, 1), (1, 1)],   // 90°
                vec![(-1, 0), (0, 0), (1, 0), (1, -1)],  // 180°
                vec![(-1, -1), (0, -1), (0, 0), (0, 1)], // 270°
            ],
            PieceType::L => vec![
                vec![(-1, 0), (0, 0), (1, 0), (1, 1)],   // 0°
                vec![(0, -1), (0, 0), (0, 1), (1, -1)],  // 90°
                vec![(-1, -1), (-1, 0), (0, 0), (1, 0)], // 180°
                vec![(0, -1), (0, 0), (0, 1), (-1, 1)],  // 270°
            ],
            PieceType::S => vec![
                vec![(0, 0), (1, 0), (-1, 1), (0, 1)], // 0°
                vec![(0, -1), (0, 0), (1, 0), (1, 1)], // 90°
                vec![(0, 0), (1, 0), (-1, 1), (0, 1)], // 180°
                vec![(0, -1), (0, 0), (1, 0), (1, 1)], // 270°
            ],
            PieceType::Z => vec![
                vec![(-1, 0), (0, 0), (0, 1), (1, 1)], // 0°
                vec![(1, -1), (0, 0), (1, 0), (0, 1)], // 90°
                vec![(-1, 0), (0, 0), (0, 1), (1, 1)], // 180°
                vec![(1, -1), (0, 0), (1, 0), (0, 1)], // 270°
            ],
            PieceType::T => vec![
                vec![(-1, 0), (0, 0), (1, 0), (0, 1)],  // 0°
                vec![(0, -1), (0, 0), (1, 0), (0, 1)],  // 90°
                vec![(-1, 0), (0, 0), (1, 0), (0, -1)], // 180°
                vec![(0, -1), (0, 0), (-1, 0), (0, 1)], // 270°
            ],
            PieceType::O => vec![
                vec![(0, 0), (1, 0), (0, 1), (1, 1)], // All rotations are the same
            ],
        };

        Self { typ, rotations }
    }
}
