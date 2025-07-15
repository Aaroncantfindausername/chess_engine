use crate::chess_board::{self, *};
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

#[derive(Component)]
pub struct HighlightSquare;

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
        .init_resource::<MovableSquares>()
        .add_event::<PiecePress>()
        .add_event::<PieceRelease>()
        .add_systems(Startup, (setup_camera, setup_board))
        .add_systems(
            Update,
            (
                update_cursor_system,
                square_press_system,
                square_drag_system,
                movable_squares_system,
            )
                .chain(),
        );
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Event)]
pub struct PiecePress;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Event)]
pub struct PieceRelease;

#[derive(Debug, Resource, Default, Clone, PartialEq, Eq)]
pub struct MovableSquares(Option<Vec<BoardPos>>);

impl MovableSquares {
    fn get(&self) -> Option<Vec<BoardPos>> {
        self.0.clone()
    }
    fn set(&mut self, target: Option<Vec<BoardPos>>) {
        self.0 = target;
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

    for idx in (0..64) {
        const HIGHTLIGHT_SQ_PATH: &str = "king_white.png";
        commands.spawn((
            Sprite::from_image(asset_server.load(HIGHTLIGHT_SQ_PATH)),
            Transform::from_translation(board_sq_pos[idx].with_z(-1.0)).with_scale(Vec3 {
                x: 0.1,
                y: 0.1,
                z: 1.0,
            }),
            BoardPos::new(idx as u8),
            HighlightSquare,
        ));
    }
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, MainCamera));
}

pub fn square_press_system(
    mut is_drag: ResMut<DragPiece>,
    mut writer: EventWriter<PiecePress>,
    piece_query: Query<(&BoardPos, &Piece), With<Piece>>,
    mut movable_sqs: ResMut<MovableSquares>,
    buttons: Res<ButtonInput<MouseButton>>,
    cursor_cords: Res<WorldCursorCoords>,
    board: Res<Board>,
) {
    let mut target_idx: Option<BoardPos> = None;
    let turn = board.get_turn();
    if buttons.just_pressed(MouseButton::Left) {
        if let Some(idx) = cords_to_index(cursor_cords.get()) {
            if board[idx]
                .get_piece()
                .is_some_and(|piece| piece.get_colour() == turn)
            {
                // Pressed piece
                eprintln!("Pressed on movable piece with index: {idx:?}");
                eprintln!(
                    "Movablesquares added: {:?}",
                    board.legal_movable_squares(idx)
                );
                // Display legal moves
                movable_sqs.set(Some(board.legal_movable_squares(idx)));
                writer.write(PiecePress);
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

pub fn gui_make_legal_move(
    board: &Board,
    piece_query: Query<(&mut BoardPos, &mut Transform), (With<BoardPos>, Without<HighlightSquare>)>,
    mv: LegalMove,
) {
    eprintln!("gui making legal move");
    let from = mv.to();
    let to = mv.from();
    if let Square(Some(piece)) = board[from] {
        use Colour::*;
        use PieceKind::*;
        match piece {
            Piece {
                kind: King,
                colour: White,
            } => {
                // Check for short castle white
                if to == BoardPos::G1 && from == BoardPos::E1 {
                    // Move rook
                    board[BoardPos::F1] = sq!(Rook, White);
                    board[BoardPos::H1] = sq!();
                } else if to.0 == 2 && from.0 == 4 {
                    // long castle white
                    // Move rook
                    board[BoardPos::D1] = sq!(Rook, White);
                    board[BoardPos::A1] = sq!();
                }
            }
            Piece {
                kind: King,
                colour: Black,
            } => {
                // Check for short castle black
                if to == BoardPos::G8 && from == BoardPos::E8 {
                    // Move rook
                    board[BoardPos::F8] = sq!(Rook, Black);
                    board[BoardPos::H8] = sq!();
                } else if to == BoardPos::C8 && from == BoardPos::E8 {
                    // long castle black
                    // Move rook
                    board[BoardPos::D8] = sq!(Rook, Black);
                    board[BoardPos::A8] = sq!();
                }
                board.king_pos.set(to, board.get_turn());
            }
            Piece { kind: Pawn, colour } => {
                let from_file = from.get_column();
                let to_file = to.get_column();
                // En passant: diagonal move but target square is empty
                if (to_file != from_file) && board[to].0.is_none() {
                    let captured_pawn_pos = match colour {
                        Black => to.0 - 8,
                        White => to.0 + 8,
                    };
                    board[BoardPos(captured_pawn_pos)] = sq!();
                } else if (to_file, colour) == (7, White) || (to_file, colour) == (0, Black) {
                    let kind =
                        mv.0.promotion_type
                            .expect("No promotion_type given for move");
                    board[to] = Square(Some(Piece { kind, colour }));
                    board[from] = sq!();
                    return board;
                }
            }
            _ => {}
        }
        board[to] = Square(Some(piece));
        board[from] = sq!();
    } else {
        panic!("Legal move on non existant square");
    }
}

pub fn square_drag_system(
    mut is_drag: ResMut<DragPiece>,
    mut board: ResMut<Board>,
    board_sq_pos: Res<BoardSqPos>,
    buttons: Res<ButtonInput<MouseButton>>,
    cursor_cords: Res<WorldCursorCoords>,
    mut writer: EventWriter<PieceRelease>,
    piece_query: Query<(&mut BoardPos, &mut Transform), (With<BoardPos>, Without<HighlightSquare>)>,
    movable_sqs: Res<MovableSquares>,
) {
    let turn = board.get_turn();
    let mut released = false;
    if let Some(pos) = is_drag.get() {
        if buttons.just_released(MouseButton::Left) {
            is_drag.set(None);
            eprintln!("Released from drag");
            released = true;
            // Stop displaying squares to move
            writer.write(PieceRelease);
        }

        for (mut idx, mut transform) in piece_query {
            if *idx == pos {
                if released {
                    eprintln!("Released and Snapping to grid");
                    let target_index = cords_to_index(transform.translation);
                    let vec;
                    if let Some(target_pos) = target_index {
                        if movable_sqs
                            .get()
                            .expect("Should have movable sqs if just dragging piece")
                            .contains(&target_pos)
                        {
                            *board = board.make_legal_move(LegalMove::new_unchecked(
                                chess_board::Move::new(*idx, target_pos, None),
                            ));
                            vec = board_sq_pos[target_pos];
                            *idx = target_pos;
                            // TODO check if castle and if so move update rook pos
                            // TODO ask for promotion type if pawn move to promotion row
                        } else {
                            // don't make move and snap piece back to original square
                            vec = board_sq_pos[*idx]
                        }
                    } else {
                        vec = board_sq_pos[*idx]
                    }
                    *transform = transform.with_translation(vec);
                    gui_move_piece(transform.deref_mut(), vec, &board_sq_pos);
                } else {
                    gui_move_piece(
                        transform.deref_mut(),
                        cursor_cords.deref().get(),
                        &board_sq_pos,
                    );
                }
            }
        }
    }
}

fn gui_move_piece(transform: &mut Transform, target_vec: Vec3, board_sq_pos: &BoardSqPos) {
    *transform = transform.with_translation(target_vec.with_z(1.0));
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
    // then, ask bevy to convert into world coordinates
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

pub fn movable_squares_system(
    coords: Res<WorldCursorCoords>,
    board: Res<Board>,
    board_sq_pos: Res<BoardSqPos>,
    mut press_reader: EventReader<PiecePress>,
    mut release_reader: EventReader<PieceRelease>,
    mut movable_sqs: ResMut<MovableSquares>,
    mut highlight_query: Query<(&mut Transform, &BoardPos), With<HighlightSquare>>,
) {
    let mut toggle_highlight_squares = |sqs: &Vec<BoardPos>| {
        eprintln!("toggling squares: {sqs:?}");
        for (mut transform, idx) in highlight_query.iter_mut() {
            eprintln!("idx in query: {idx:?}");
            if sqs.contains(idx) {
                if transform.deref().translation.z == -1.0 {
                    eprintln!("toggle_highlight_squares: making square {idx:?} visible");
                    transform.deref_mut().translation.z = 2.0;
                } else if transform.deref().translation.z == 2.0 {
                    eprintln!("toggle_highlight_squares: making square {idx:?} hidden");
                    transform.deref_mut().translation.z = -1.0;
                } else {
                    panic!("HighlightSquare transform something other than -1 or 2.0")
                }
            }
        }
    };
    // TODO remove if press and release don't happen simul
    if !press_reader.is_empty() && !release_reader.is_empty() {
        panic!("pressed and released on the some frame");
    }
    if !press_reader.is_empty() {
        eprintln!("received press event");
        if press_reader.len() > 1 {
            panic!("More than one event in press_reader");
        }
        for _ in press_reader.read() {
            toggle_highlight_squares(
                &movable_sqs
                    .deref()
                    .get()
                    .expect("If just pressed, movable_sqs shouldn't be empty"),
            );
        }
    } else if !release_reader.is_empty() {
        eprintln!("received release event");
        toggle_highlight_squares(
            &movable_sqs
                .0
                .clone()
                .expect("MovableSquares shouldn't be none after just releasing"),
        );
        release_reader.clear();
        movable_sqs.set(None);
    }
}
