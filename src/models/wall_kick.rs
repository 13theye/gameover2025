// src/models/wall_kick.rs
//
// Table describing possible wall kick positions for piece rotations

pub type WallKickOffset = (isize, isize); // (dx, dy)

// For J, L, S, Z, T pieces (standard SRS kicks)
pub const JLSZT_WALL_KICKS: [&[WallKickOffset]; 8] = [
    // 0>>1 (0° to 90° clockwise)
    &[(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
    // 1>>0 (90° to 0° counter-clockwise)
    &[(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
    // 1>>2 (90° to 180° clockwise)
    &[(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
    // 2>>1 (180° to 90° counter-clockwise)
    &[(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
    // 2>>3 (180° to 270° clockwise)
    &[(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
    // 3>>2 (270° to 180° counter-clockwise)
    &[(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
    // 3>>0 (270° to 0° clockwise)
    &[(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
    // 0>>3 (0° to 270° counter-clockwise)
    &[(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
];

// For I piece (has different offsets due to its shape)
pub const I_WALL_KICKS: [&[WallKickOffset]; 8] = [
    // 0>>1
    &[(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
    // 1>>0
    &[(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
    // 1>>2
    &[(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
    // 2>>1
    &[(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
    // 2>>3
    &[(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
    // 3>>2
    &[(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
    // 3>>0
    &[(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
    // 0>>3
    &[(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
];

// O piece never needs wall kicks
pub const O_WALL_KICKS: [&[WallKickOffset]; 1] = [&[(0, 0)]];

// Helper to get the correct index for wall kick data
pub fn get_wall_kick_index(from_rot: usize, to_rot: usize) -> usize {
    match (from_rot % 4, to_rot % 4) {
        (0, 1) => 0, // 0>>1 (clockwise)
        (1, 0) => 1, // 1>>0 (counter-clockwise)
        (1, 2) => 2, // 1>>2 (clockwise)
        (2, 1) => 3, // 2>>1 (counter-clockwise)
        (2, 3) => 4, // 2>>3 (clockwise)
        (3, 2) => 5, // 3>>2 (counter-clockwise)
        (3, 0) => 6, // 3>>0 (clockwise)
        (0, 3) => 7, // 0>>3 (counter-clockwise)
        _ => 0,      // Fallback
    }
}
