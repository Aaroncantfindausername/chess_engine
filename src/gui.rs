use crate::chess_board::*;
use bevy::ecs::query;
use bevy::{
    ecs::{component::Component, event::Event},
    math::Vec3,
    window::PrimaryWindow,
};
use bevy::{prelude::*, transform};
use std::ops::{Index, IndexMut};

use std::ops::{Deref, DerefMut};
use std::process::id;
use std::u8;

pub const BOARD_SCALE: f32 = 0.8;
const BOARD_SIZE: f32 = 1166.0 * BOARD_SCALE;
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
        pos.x = (column - 3.5) * (SQUARE_SIZE);
        pos.y = (row - 3.5) * (SQUARE_SIZE);
        pos.z = 1.0;
    }
    pos_array
}

pub fn gui_plugin(app: &mut App) {
    app.init_resource::<Board>()
        .init_resource::<WorldCursorCoords>()
        .init_resource::<DragPiece>()
        .init_resource::<BoardSqPos>()
        .init_resource::<ToMove>()
        .add_systems(Startup, (setup_camera, setup_board))
        .add_systems(
            Update,
            (
                update_cursor_system,
                square_press_system,
                square_drag_system,
            )
                .chain(),
        );
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Event)]
pub struct ButtonPress(BoardPos);

impl ButtonPress {
    fn new(board_pos: BoardPos) -> Self {
        ButtonPress(board_pos)
    }
}

#[derive(Debug, Resource, Default, Clone)]
pub struct WorldCursorCoords(Vec3);

impl WorldCursorCoords {
    fn get(&self) -> Vec3 {
        self.0
    }

    fn set(&mut self, val: Vec3) {
        self.0 = val;
    }
}

#[derive(Debug, Resource, Default, PartialEq, Eq, Hash)]
pub struct DragPiece(Option<BoardPos>);

impl DragPiece {
    fn new(pos: Option<BoardPos>) -> Self {
        DragPiece(pos)
    }
}

#[derive(Debug, Resource, Clone)]
pub struct BoardSqPos([Vec3; 64]);

impl Index<BoardPos> for BoardSqPos {
    type Output = Vec3;
    fn index(&self, index: BoardPos) -> &Self::Output {
        &self.0[usize::from(index)]
    }
}

impl Default for BoardSqPos {
    fn default() -> Self {
        BoardSqPos(calc_sq_pos())
    }
}

impl DragPiece {
    fn get(&self) -> Option<BoardPos> {
        self.0
    }

    fn set(&mut self, val: Option<BoardPos>) {
        self.0 = val;
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
                z: 0.0,
            }),
        ))
        .id();
    let board_sq_pos = board_sq_pos.0;
    for idx in (0..16).chain(48..64) {
        let piece = board.deref()[BoardPos::new(idx as u8)]
            .get_piece()
            .expect("Expected piece, found none instead while calculating path to piece imgs");
        let path = piece.to_string() + ".png";
        let child_piece = commands
            .spawn((
                Sprite::from_image(asset_server.load(path)),
                Transform::from_translation(board_sq_pos[idx]).with_scale(Vec3::splat(0.15)),
                piece,
                BoardPos::new(idx as u8),
            ))
            .id();
        //commands.entity(root_board).add_child(child_piece);
    }
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, MainCamera));
}

pub fn square_press_system(
    mut is_drag: ResMut<DragPiece>,
    turn: Res<ToMove>,
    piece_query: Query<(&BoardPos, &Piece), With<Piece>>,
    board_sq_pos: Res<BoardSqPos>,
    buttons: Res<ButtonInput<MouseButton>>,
    cursor_cords: Res<WorldCursorCoords>,
    board: Res<Board>,
) {
    let mut target_idx: Option<BoardPos> = None;
    if buttons.just_pressed(MouseButton::Left) {
        if let Some(idx) = cords_to_index(cursor_cords.get()) {
            if board[idx]
                .get_piece()
                .is_some_and(|piece| piece.get_colour() == **turn)
            {
                // Pressed piece
                eprintln!("Pressed on movable piece with index: {idx:?}");
                target_idx = Some(idx);
            }
        }
    }

    if target_idx.is_none() {
        return;
    }

    for (&idx, &piece) in piece_query {
        // If piece pressed and that piece is the correct colour, start dragging it
        if target_idx
            .expect("target_piece shouldn't be None as would've already returned from function")
            == idx
        {
            is_drag.set(Some(idx));
            eprintln!("Started dragging {piece:?} with idx {idx:?}");
        }
    }
}

fn cords_to_index(pos: Vec3) -> Option<BoardPos> {
    if (-BOARD_SIZE / 2.0..=BOARD_SIZE / 2.0).contains(&pos.x)
        && (-BOARD_SIZE / 2.0..=BOARD_SIZE / 2.0).contains(&pos.y)
    {
        let column = ((pos.x + BOARD_SIZE / 2.0) / SQUARE_SIZE).floor();
        let row = ((pos.y + BOARD_SIZE / 2.0) / SQUARE_SIZE).floor();
        let pos = Some(BoardPos::new((row * 8.0 + column) as u8));
        eprintln!("cords to index returning press INSIDE of board: {pos:?}");
        pos
    } else {
        eprintln!("cords to index returning press OUTSIDE of board, coords: {pos}");
        None
    }
}

pub fn square_drag_system(
    mut is_drag: ResMut<DragPiece>,
    mut turn: ResMut<ToMove>,
    mut board: ResMut<Board>,
    board_sq_pos: Res<BoardSqPos>,
    buttons: Res<ButtonInput<MouseButton>>,
    cursor_cords: Res<WorldCursorCoords>,
    mut piece_query: Query<(&mut BoardPos, &mut Transform), With<BoardPos>>,
) {
    let mut released = false;
    if let Some(pos) = is_drag.get() {
        if buttons.just_released(MouseButton::Left) {
            is_drag.set(None);
            eprintln!("Released from drag");
            released = true;
        }

        for (mut idx, mut transform) in piece_query {
            if *idx == pos {
                if released {
                    eprintln!("Released and Snapping to grid");
                    let target_index = cords_to_index(transform.translation);
                    let vec;
                    if let Some(pos) = target_index {
                        vec = board_sq_pos[pos];
                        *idx = pos;
                        match **turn {
                            Colour::Black => *turn = ToMove::new(Colour::White),
                            Colour::White => *turn = ToMove::new(Colour::Black),
                        }
                    } else {
                        vec = board_sq_pos[*idx]
                    }
                    *transform = transform.with_translation(vec);
                } else {
                    eprintln!(
                        "square_drag_system - unreleased and dragging, cursor coords: {:?}, current coords: {:?}",
                        cursor_cords.deref().get(), transform.translation
                    );
                    *transform = transform
                        .deref()
                        .with_translation(cursor_cords.deref().get().with_z(1.0));
                }
            }
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
        .map(|ray| ray.origin)
    {
        coords.set(world_pos);
        // eprintln!("Setting cursor coords to {world_pos:?}");
    }
}
