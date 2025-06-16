use crate::chess_board::*;
use bevy::math::Vec3;

pub const BOARD_SCALE: f32 = 0.8;
const BOARD_SIZE: f32 = 1166.0;

fn calc_sq_pos() -> [Vec3; 64] {
    let pos_array: [Vec3; 64] = [Vec3::default(); 64];
    pos_array
}
