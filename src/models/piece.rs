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

// Type alias for a Tetromino block
type Block = (i8, i8);

#[derive(Debug)]
pub struct Piece {
    pub typ: PieceType,
    pub rotations: &'static [[Block; 4]],
}

impl Piece {
    pub fn new(typ: PieceType) -> Self {
        let rotations = match typ {
            PieceType::I => &I_ROTATIONS,
            PieceType::J => &J_ROTATIONS,
            PieceType::L => &L_ROTATIONS,
            PieceType::S => &S_ROTATIONS,
            PieceType::Z => &Z_ROTATIONS,
            PieceType::T => &T_ROTATIONS,
            PieceType::O => &O_ROTATIONS,
        };

        Self { typ, rotations }
    }
}

/******************* Piece Rotation Definitions ******************/

// (0,0) is the central pivot point

const I_ROTATIONS: [[Block; 4]; 4] = [
    [(-1, 0), (0, 0), (1, 0), (2, 0)], // 0 deg
    [(0, -1), (0, 0), (0, 1), (0, 2)], // 90 deg
    [(-1, 0), (0, 0), (1, 0), (2, 0)], // 180 deg
    [(0, -1), (0, 0), (0, 1), (0, 2)], // 270 deg
];

const J_ROTATIONS: [[Block; 4]; 4] = [
    [(-1, 0), (0, 0), (1, 0), (-1, 1)],  // 0°
    [(0, -1), (0, 0), (0, 1), (1, 1)],   // 90°
    [(-1, 0), (0, 0), (1, 0), (1, -1)],  // 180°
    [(-1, -1), (0, -1), (0, 0), (0, 1)], // 270°
];

const L_ROTATIONS: [[Block; 4]; 4] = [
    [(-1, 0), (0, 0), (1, 0), (1, 1)],   // 0°
    [(0, -1), (0, 0), (0, 1), (1, -1)],  // 90°
    [(-1, -1), (-1, 0), (0, 0), (1, 0)], // 180°
    [(0, -1), (0, 0), (0, 1), (-1, 1)],  // 270°
];

const S_ROTATIONS: [[Block; 4]; 4] = [
    [(0, 0), (1, 0), (-1, 1), (0, 1)], // 0°
    [(0, -1), (0, 0), (1, 0), (1, 1)], // 90°
    [(0, 0), (1, 0), (-1, 1), (0, 1)], // 180°
    [(0, -1), (0, 0), (1, 0), (1, 1)], // 270°
];

const Z_ROTATIONS: [[Block; 4]; 4] = [
    [(-1, 0), (0, 0), (0, 1), (1, 1)], // 0°
    [(1, -1), (0, 0), (1, 0), (0, 1)], // 90°
    [(-1, 0), (0, 0), (0, 1), (1, 1)], // 180°
    [(1, -1), (0, 0), (1, 0), (0, 1)], // 270°
];

const T_ROTATIONS: [[Block; 4]; 4] = [
    [(-1, 0), (0, 0), (1, 0), (0, 1)],  // 0°
    [(0, -1), (0, 0), (1, 0), (0, 1)],  // 90°
    [(0, -1), (0, 0), (-1, 0), (0, 1)], // 270°
    [(-1, 0), (0, 0), (1, 0), (0, -1)], // 180°
];

const O_ROTATIONS: [[Block; 4]; 4] = [
    [(0, 0), (1, 0), (0, 1), (1, 1)],
    [(0, 0), (1, 0), (0, 1), (1, 1)],
    [(0, 0), (1, 0), (0, 1), (1, 1)],
    [(0, 0), (1, 0), (0, 1), (1, 1)], // All rotations are the same
];
