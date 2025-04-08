// src/models/piece.rs
//
//
// Defining the Tetris piece model
//
// A Piece's (0,0) is the central pivot point of rotation

// Type alias for a Tetromino block
type Block = (isize, isize);

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

impl PieceType {
    pub const fn get_rotations(&self) -> &'static [[Block; 4]; 4] {
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

    /******************* Piece Sizing Methods ********************/

    pub const fn y_min(&self, rotation_index: usize) -> isize {
        let rotation = self.get_rotation(rotation_index);

        // Initialize with the first block's y-coordinate
        let mut lowest = isize::MAX;

        // Manual iteration for const context
        let mut i = 0;
        while i < self.rotation_count() {
            if rotation[i].1 < lowest {
                lowest = rotation[i].1;
            }
            i += 1;
        }
        lowest
    }

    pub const fn x_min(&self, rotation_index: usize) -> isize {
        let rotation = self.get_rotation(rotation_index);

        // Initialize with the first block's y-coordinate
        let mut min = isize::MAX;

        // Manual iteration for const context
        let mut i = 0;
        while i < self.rotation_count() {
            if rotation[i].0 < min {
                min = rotation[i].0;
            }
            i += 1;
        }
        min
    }

    pub const fn x_max(&self, rotation_index: usize) -> isize {
        let rotation = self.get_rotation(rotation_index);

        // Initialize with the first block's y-coordinate
        let mut max = isize::MIN;

        // Manual iteration for const context
        let mut i = 0;
        while i < self.rotation_count() {
            if rotation[i].0 < max {
                max = rotation[i].0;
            }
            i += 1;
        }
        max
    }

    /******************* Piece Rotation Methods ******************/

    pub const fn get_rotation(&self, rotation_index: usize) -> &'static [Block; 4] {
        &self.get_rotations()[rotation_index % self.rotation_count()]
    }

    pub const fn rotation_count(&self) -> usize {
        self.get_rotations().len()
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
