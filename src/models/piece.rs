// src/models/piece.rs
//
//
// Defining the Tetris piece model
//
// A Piece's (0,0) is the bottom-left corner of the piece's footprint

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

    // returns a vec where each index is a dx relative to min_x
    // and value is the lowest y-value for that x-coordinate
    pub fn skirt(&self, rot_idx: usize) -> Vec<isize> {
        let piece = self.get_rotation(rot_idx);

        // Find min/max x to determine skirt width
        let (min_x, max_x) = self.minmax_x(rot_idx);

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

    // returns the minimum and maximum x offsets
    pub fn minmax_x(&self, rot_idx: usize) -> (isize, isize) {
        let piece = self.get_rotation(rot_idx);
        (
            piece.iter().map(|&(x, _)| x).min().unwrap(),
            piece.iter().map(|&(x, _)| x).max().unwrap(),
        )
    }

    // returns the maximum x offset
    pub fn max_x(&self, rot_idx: usize) -> isize {
        let piece = self.get_rotation(rot_idx);
        piece.iter().map(|&(x, _)| x).max().unwrap()
    }

    // returns the highest y offset
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

// bottom-left origin

const I_ROTATIONS: [[Block; 4]; 4] = [
    [(0, 0), (1, 0), (2, 0), (3, 0)], // 0° - center is between blocks at (1.5, 0.5)
    [(2, 0), (2, 1), (2, 2), (2, 3)], // 90° - center is between blocks
    [(0, 1), (1, 1), (2, 1), (3, 1)], // 180° - center is between blocks
    [(1, 0), (1, 1), (1, 2), (1, 3)], // 270° - center is between blocks
];

const J_ROTATIONS: [[Block; 4]; 4] = [
    [(0, 0), (0, 1), (1, 1), (2, 1)], // 0°
    [(1, 0), (2, 0), (1, 1), (1, 2)], // 90°
    [(0, 1), (1, 1), (2, 1), (2, 2)], // 180°
    [(1, 0), (1, 1), (0, 2), (1, 2)], // 270°
];

const L_ROTATIONS: [[Block; 4]; 4] = [
    [(0, 1), (1, 1), (2, 1), (2, 0)], // 0°
    [(1, 0), (1, 1), (1, 2), (2, 2)], // 90°
    [(0, 1), (0, 2), (1, 1), (2, 1)], // 180°
    [(0, 0), (1, 0), (1, 1), (1, 2)], // 270°
];

const S_ROTATIONS: [[Block; 4]; 4] = [
    [(1, 0), (2, 0), (0, 1), (1, 1)], // 0°
    [(1, 0), (1, 1), (2, 1), (2, 2)], // 90°
    [(1, 1), (2, 1), (0, 2), (1, 2)], // 180°
    [(0, 0), (0, 1), (1, 1), (1, 2)], // 270°
];

const Z_ROTATIONS: [[Block; 4]; 4] = [
    [(0, 0), (1, 0), (1, 1), (2, 1)], // 0°
    [(2, 0), (1, 1), (2, 1), (1, 2)], // 90°
    [(0, 1), (1, 1), (1, 2), (2, 2)], // 180°
    [(1, 0), (0, 1), (1, 1), (0, 2)], // 270°
];

const T_ROTATIONS: [[Block; 4]; 4] = [
    [(0, 1), (1, 1), (2, 1), (1, 0)], // 0°
    [(1, 0), (1, 1), (1, 2), (2, 1)], // 90°
    [(0, 1), (1, 1), (2, 1), (1, 2)], // 180°
    [(0, 1), (1, 0), (1, 1), (1, 2)], // 270°
];

const O_ROTATIONS: [[Block; 4]; 4] = [
    [(0, 0), (1, 0), (0, 1), (1, 1)], // All rotations are the same
    [(0, 0), (1, 0), (0, 1), (1, 1)],
    [(0, 0), (1, 0), (0, 1), (1, 1)],
    [(0, 0), (1, 0), (0, 1), (1, 1)],
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piece_skirt() {
        // Test each piece type
        let pieces = [
            PieceType::I,
            PieceType::J,
            PieceType::L,
            PieceType::S,
            PieceType::Z,
            PieceType::T,
            PieceType::O,
        ];

        for piece_type in pieces.iter() {
            for rot_idx in 0..piece_type.rotation_count() {
                let rotation = piece_type.get_rotation(rot_idx);
                let skirt = piece_type.skirt(rot_idx);

                // Find the min/max x manually to verify
                let min_x = rotation.iter().map(|&(x, _)| x).min().unwrap();
                let max_x = rotation.iter().map(|&(x, _)| x).max().unwrap();

                // Verify skirt length
                let expected_width = (max_x - min_x + 1) as usize;
                assert_eq!(
                    skirt.len(),
                    expected_width,
                    "Incorrect skirt width for {:?} rotation {}",
                    piece_type,
                    rot_idx
                );

                // Verify each skirt value
                for x in min_x..=max_x {
                    let rel_x = (x - min_x) as usize;

                    // Calculate expected min y for this x
                    let expected_min_y = rotation
                        .iter()
                        .filter(|&&(cell_x, _)| cell_x == x)
                        .map(|&(_, cell_y)| cell_y)
                        .min();

                    // If there's no cell at this x, the skirt should be isize::MAX
                    // Otherwise, it should be the minimum y
                    if let Some(min_y) = expected_min_y {
                        assert_eq!(
                            skirt[rel_x], min_y,
                            "Incorrect skirt value for {:?} rotation {} at x={}",
                            piece_type, rot_idx, x
                        );
                    } else {
                        assert_eq!(
                            skirt[rel_x],
                            isize::MAX,
                            "Expected MAX skirt value for {:?} rotation {} at x={}",
                            piece_type,
                            rot_idx,
                            x
                        );
                    }
                }

                // Print the test results for debugging
                println!("Piece: {:?}, Rotation: {}", piece_type, rot_idx);
                println!("Cells: {:?}", rotation);
                println!("Skirt: {:?}", skirt);
                println!("-----");
            }
        }
    }
}
