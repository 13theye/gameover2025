// src/models/piece.rs
//
//
// Defining the Tetris piece model
//
// A Piece's (0,0) is the central pivot point of rotation

// Type alias for a Tetromino block
type Block = (isize, isize);

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PieceType {
    I,
    J,
    L,
    S,
    Z,
    T,
    O,
}

impl PieceType {
    pub const fn rotations(&self) -> &'static [[Block; 4]; 4] {
        match self {
            PieceType::I => &I_ROTATIONS,
            PieceType::J => &J_ROTATIONS,
            PieceType::L => &L_ROTATIONS,
            PieceType::S => &S_ROTATIONS,
            PieceType::Z => &Z_ROTATIONS,
            PieceType::T => &T_ROTATIONS,
            PieceType::O => &O_ROTATIONS,
        }
    }

    // returns a vec where each index is an x-coordinate
    // and value is the lowest y-value for that x-coordinate
    pub fn skirt(&self, rot_idx: usize) -> Vec<isize> {
        let piece = self.get_rotation(rot_idx);

        // Find min/max x to determine skirt width
        let min_x = piece.iter().map(|&(x, _)| x).min().unwrap();
        let max_x = piece.iter().map(|&(x, _)| x).max().unwrap();

        // Initialize skirt with maximum possible y-values
        let width = (max_x - min_x + 1) as usize;
        let mut skirt = vec![isize::MAX; width];

        // Calculate lowest y for each x
        for &(x, y) in piece {
            let index = (x - min_x) as usize;
            if y < skirt[index] {
                skirt[index] = y;
            }
        }

        skirt
    }

    pub fn minmax_x(&self, rot_idx: usize) -> (isize, isize) {
        let piece = self.get_rotation(rot_idx);
        (
            piece.iter().map(|&(x, _)| x).min().unwrap(),
            piece.iter().map(|&(x, _)| x).max().unwrap(),
        )
    }

    pub fn max_y(&self, rot_idx: usize) -> isize {
        let piece = self.get_rotation(rot_idx);
        piece.iter().map(|&(_, y)| y).max().unwrap()
    }

    /******************* Utility Methods ******************/
    const ALL: [PieceType; 7] = [
        PieceType::I,
        PieceType::J,
        PieceType::L,
        PieceType::S,
        PieceType::Z,
        PieceType::T,
        PieceType::O,
    ];

    pub fn from_idx(idx: usize) -> Self {
        let safe_idx = idx % Self::ALL.len();
        Self::ALL[safe_idx]
    }

    pub const fn get_rotation(&self, rot_idx: usize) -> &'static [Block; 4] {
        &self.rotations()[rot_idx % self.rotation_count()]
    }

    pub const fn rotation_count(&self) -> usize {
        self.rotations().len()
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
    [(-1, 0), (0, 0), (1, 0), (0, -1)], // 180°
    [(0, -1), (0, 0), (-1, 0), (0, 1)], // 270°
];

const O_ROTATIONS: [[Block; 4]; 4] = [
    [(0, 0), (1, 0), (0, 1), (1, 1)],
    [(0, 0), (1, 0), (0, 1), (1, 1)],
    [(0, 0), (1, 0), (0, 1), (1, 1)],
    [(0, 0), (1, 0), (0, 1), (1, 1)], // All rotations are the same
];
