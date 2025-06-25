use crate::chess_board::*;
use bevy::ecs::query;
use bevy::prelude::*;
use bevy::{
    ecs::{component::Component, event::Event},
    math::Vec3,
    window::PrimaryWindow,
};

use std::ops::Deref;

pub const BOARD_SCALE: f32 = 0.8;
const BOARD_SIZE: f32 = 1166.0;
const SQUARE_SIZE: f32 = BOARD_SIZE / 8.0;

#[derive(Component)]
pub struct MainCamera;

pub fn calc_sq_pos() -> [Vec3; 64] {
    let mut pos_array: [Vec3; 64] = [Vec3::default(); 64];
    let mut row: f32;
    let mut column: f32;
    for (index, pos) in pos_array.iter_mut().enumerate() {
        row = (index / 8) as f32;
        column = (index % 8) as f32;
        pos.x = (column - 3.5) * SQUARE_SIZE;
        pos.y = (row - 3.5) * SQUARE_SIZE;
        pos.z = 1.0;
    }
    pos_array
}

pub fn gui_plugin(app: &mut App) {
    app.insert_resource(Board::create_standard())
        .init_resource::<WorldCursorCoords>()
        .init_resource::<DragPiece>()
        .init_resource::<BoardSqPos>()
        .add_systems(Startup, (setup_camera, setup_board));
}

#[derive(Debug, Event)]
pub struct ButtonPress(Square);

#[derive(Debug, Resource, Default)]
pub struct WorldCursorCoords(Vec2);

#[derive(Debug, Resource, Default)]
pub struct DragPiece(bool); // TODO change to Option<Square>

#[derive(Debug, Resource)]
pub struct BoardSqPos([Vec3; 64]);

impl Default for BoardSqPos {
    fn default() -> Self {
        BoardSqPos(calc_sq_pos())
    }
}

pub fn setup_board(
    mut commands: Commands,
    board_sq_pos: Res<BoardSqPos>,
    asset_server: Res<AssetServer>,
    board: Res<Board>,
) {
    let root_board = commands
        .spawn((
            Sprite::from_image(asset_server.load("chess_board.png")),
            Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3 {
                x: BOARD_SCALE,
                y: BOARD_SCALE,
                z: 1.0,
            }),
        ))
        .id();
    let board_sq_pos = board_sq_pos.0;
    for idx in (0..16).chain(48..64) {
        let piece = board.deref()[BoardPos(idx as u8)]
            .get_piece()
            .expect("Expected piece, found none instead while calculating path to piece imgs");
        let path = piece.to_string() + ".png";
        let child_piece = commands
            .spawn((
                Sprite::from_image(asset_server.load(path)),
                Transform::from_translation(board_sq_pos[idx]).with_scale(Vec3::splat(0.16)),
                Button,
                piece.kind,
            ))
            .id();
        commands.entity(root_board).add_child(child_piece);
    }
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, MainCamera));
}

pub fn square_press_system(
    mut is_drag: ResMut<DragPiece>,
    interaction_query: Query<&Interaction, (With<PieceKind>, Changed<Interaction>)>,
    board_sq_pos: Res<BoardSqPos>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    for &intercation in interaction_query {
        if intercation == Interaction::Pressed {
            is_drag.0 = true;
        }
    }
}

pub fn update_cursor_system(
    mut coords: ResMut<WorldCursorCoords>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK
    let (camera, camera_transform) = q_camera.single().expect("More than 1 camera found");
    // There is only one primary window, so we can similarly get it from the query:
    let window = q_window.single().expect("More than 1 window found");
    // check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates, and truncate to discard Z
    if let Some(world_pos) = window
        .cursor_position()
        .map(|cursor| {
            camera
                .viewport_to_world(camera_transform, cursor)
                .expect("invalid camera_transform probably")
        })
        .map(|ray| ray.origin.truncate())
    {
        coords.0 = world_pos;
    }
}
