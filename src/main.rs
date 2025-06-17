use std::ops::Deref;

use bevy::prelude::*;
use chess_board::Board;
use chess_engine::chess_board::{BoardPos, Square};
use chess_engine::gui::*;
use chess_engine::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(chess_board::Board::create_standard())
        .add_systems(Startup, (setup_camera, setup_board))
        .run();
}

fn setup_board(mut commands: Commands, asset_server: Res<AssetServer>, board: Res<Board>) {
    let root_board = commands
        .spawn((
            Sprite::from_image(asset_server.load("chess_board.png")),
            Transform::from_xyz(0.0, 0.0, -1.0).with_scale(Vec3 {
                x: BOARD_SCALE,
                y: BOARD_SCALE,
                z: 1.0,
            }),
        ))
        .id();
    let board_sq_pos = gui::calc_sq_pos();
    for idx in 0..15 {
        let piece = board.deref()[BoardPos(idx as u8)]
            .get_piece()
            .expect("Expected piece, found none instead while calculating path to piece imgs");
        let path = format!("todo");
        let child_piece = commands
            .spawn((
                Sprite::from_image(asset_server.load(path)),
                Transform::from_translation(board_sq_pos[idx]),
            ))
            .id();
        commands.entity(root_board).add_child(child_piece);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
