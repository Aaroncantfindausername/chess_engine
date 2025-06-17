use crate::chess_board::*;
use bevy::{ecs::component::Component, math::Vec3};

pub const BOARD_SCALE: f32 = 0.8;
const BOARD_SIZE: f32 = 1166.0;

pub fn calc_sq_pos() -> [Vec3; 64] {
    let mut pos_array: [Vec3; 64] = [Vec3::default(); 64];
    let mut row: usize;
    let mut column: usize;
    for (index, &mut pos) in pos_array.iter_mut().enumerate() {
        row = index / 8;
        column = index % 8;
    }
    pos_array
}
