use crate::chess_board::*;
use bevy::math::ops::{abs, powf, sqrt};
use bevy::prelude::*;
use bevy::{
    ecs::{component::Component, event::Event},
    math::Vec3,
    window::PrimaryWindow,
};

use std::ops::{Deref, DerefMut, Index};

pub const BOARD_SCALE: f32 = 0.8;
const BOARD_SIZE: f32 = 1166.0 * BOARD_SCALE;
const SQUARE_SIZE: f32 = BOARD_SIZE / 8.0;
const KNIGHT_PROMOTION_POS: Vec3 = Vec3::new(BOARD_SIZE / 2.0 + 50.0, 200.0, 1.0);
const BISHOP_PROMOTION_POS: Vec3 = Vec3::new(BOARD_SIZE / 2.0 + 50.0, 100.0, 1.0);
const ROOK_PROMOTION_POS: Vec3 = Vec3::new(BOARD_SIZE / 2.0 + 50.0, 0.0, 1.0);
const QUEEN_PROMOTION_POS: Vec3 = Vec3::new(BOARD_SIZE / 2.0 + 50.0, -100.0, 1.0);
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
        .init_resource::<PromotingMove>()
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
                promotion_move_system,
            )
                .chain(),
        );
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Event)]
pub struct PiecePress;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Event)]
pub struct PieceRelease;

#[derive(Debug, Resource, Default, Clone)]
pub struct PromotingMove(Option<LegalMove>);

impl PromotingMove {
    fn get(&self) -> Option<LegalMove> {
        self.0
    }
    fn set(&mut self, target: Option<LegalMove>) {
        self.0 = target;
    }
}

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

#[derive(Debug, Component, Default, Clone)]
pub struct PromotionPieceMarker;

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
    let mut bundles = Vec::new();
    // Create bundles
    for idx in (0..16).chain(48..64) {
        let piece = board.deref()[BoardPos::new(idx as u8)]
            .get_piece()
            .expect("Expected piece, found none instead while calculating path to piece imgs");
        let path = piece.to_string() + ".png";
        let bundle = (
            Sprite::from_image(asset_server.load(path)),
            Transform::from_translation(board_sq_pos[idx]).with_scale(Vec3::splat(0.15)),
            piece,
            BoardPos::new(idx as u8),
        );
        bundles.push(bundle);
    }
    // Make piece entities with bundles
    commands.spawn_batch(bundles);

    for idx in 0..64 {
        const HIGHTLIGHT_SQ_PATH: &str = "grey_circle.png";
        commands.spawn((
            Sprite::from_image(asset_server.load(HIGHTLIGHT_SQ_PATH)),
            Transform::from_translation(board_sq_pos[idx].with_z(-1.0)).with_scale(Vec3 {
                x: 0.18,
                y: 0.18,
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
    paused: Res<PromotingMove>,
) {
    if paused.deref().get().is_some() {
        // Game paused, waiting for promotion type
        return;
    }
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
    mut piece_query: Query<
        (
            Entity,
            &mut BoardPos,
            &mut Transform,
            &mut Piece,
            &mut Sprite,
        ),
        (With<BoardPos>, Without<HighlightSquare>),
    >,
    mv: LegalMove,
    board_sq_pos: &BoardSqPos,
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
) {
    eprintln!("gui making legal move");
    // If move is castle
    if let Some(castle_type) = board.move_castle_type(&mv) {
        eprintln!("gui_make_legal_move; castle move detected");
        let offset: BoardPos = match board.get_turn() {
            Colour::Black => BoardPos::new(56),
            Colour::White => BoardPos::new(0),
        };
        let rook_pos: BoardPos;
        let target_rook_pos: BoardPos;
        match castle_type {
            CastleType::Long => {
                rook_pos = offset + BoardPos::A1;
                target_rook_pos = offset + BoardPos::D1;
            }
            CastleType::Short => {
                rook_pos = offset + BoardPos::H1;
                target_rook_pos = offset + BoardPos::F1
            }
        }
        // move rook to castled position

        let mut rook_transform = None;
        let mut rook_idx_component = None;
        for (_, pos, transform, _, _) in piece_query.iter_mut() {
            if *pos == rook_pos {
                rook_transform = Some(transform);
                rook_idx_component = Some(pos);
            }
        }
        let rook_transform = rook_transform.expect("Should have rook if castling");
        let rook_idx_component = rook_idx_component.expect("Should have rook if castling");
        gui_move_piece(
            rook_transform,
            rook_idx_component,
            target_rook_pos,
            board_sq_pos,
        )
    } else if let Some(en_passant_direction) = board.move_en_passant_type(&mv) {
        // En passant move
        eprintln!("Gui has identified en_passant move");
        let pawn_to_remove_pos: BoardPos = match en_passant_direction {
            EnPassantDirection::Left => mv
                .from()
                .try_move_left()
                .expect("Should have piece to the left side if en passanting left"),
            EnPassantDirection::Right => mv
                .from()
                .try_move_right()
                .expect("Should have piece to the left side if en passanting left"),
        };
        let entity_to_remove = piece_query
            .iter()
            .filter_map(|(entity, pos, _, _, _)| {
                if *pos == pawn_to_remove_pos {
                    Some(entity)
                } else {
                    None
                }
            })
            .next()
            .expect("Pawn to remove should exist if legal en passant move");
        eprintln!("despawning entity {entity_to_remove:?}");
        commands.entity(entity_to_remove).despawn();
    } else if board[mv.to()].get_piece().is_some() {
        // Capturing move
        let captured_piece_entity = piece_query
            .iter_mut()
            .filter_map(|(entity, idx, _, _, _)| {
                if *idx == mv.to() {
                    // return captured piece
                    Some(entity)
                } else {
                    None
                }
            })
            .next()
            .expect("Should be capturing piece");
        commands.entity(captured_piece_entity).despawn();
    }
    if board.is_promotion_move(&mv) {
        eprintln!("identified promotion move");
        // promotion move
        let (mut piece_comp, mut sprite) = piece_query
            .iter_mut()
            .filter_map(|(_, pos, _, piece_component, sprite)| {
                if *pos == mv.from() {
                    Some((piece_component, sprite))
                } else {
                    None
                }
            })
            .next()
            .expect("Piece should exist in query if moving it");
        // Change piece kind
        *piece_comp = Piece::new(
            mv.get_promotion_kind()
                .expect("move should have promotion type"),
            piece_comp.get_colour(),
        );
        // Change piece sprite
        eprintln!("Updated piece component{:?}", *piece_comp);
        let path = piece_comp.to_string() + ".png";
        *sprite = Sprite::from_image(asset_server.load(path));
    }

    // Make normal move
    let mut piece_transform = None;
    let mut piece_idx_component = None;
    for (_, pos, transform, _, _) in piece_query.iter_mut() {
        if *pos == mv.from() {
            piece_transform = Some(transform);
            piece_idx_component = Some(pos);
        }
    }
    let piece_transform = piece_transform.expect("Piece should exist if moving it");
    let piece_idx_component = piece_idx_component.expect("Piece should exist if moving it");
    gui_move_piece(piece_transform, piece_idx_component, mv.to(), board_sq_pos)
}

pub fn square_drag_system(
    mut is_drag: ResMut<DragPiece>,
    mut board: ResMut<Board>,
    board_sq_pos: Res<BoardSqPos>,
    buttons: Res<ButtonInput<MouseButton>>,
    cursor_cords: Res<WorldCursorCoords>,
    mut writer: EventWriter<PieceRelease>,
    mut piece_query: Query<
        (
            Entity,
            &mut BoardPos,
            &mut Transform,
            &mut Piece,
            &mut Sprite,
        ),
        (With<BoardPos>, Without<HighlightSquare>),
    >,
    movable_sqs: Res<MovableSquares>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut promotion_type: ResMut<PromotingMove>,
) {
    if promotion_type.deref().get().is_some() {
        // Game paused, waiting for promotion type
        return;
    }
    let mut released = false;
    if let Some(original_pos) = is_drag.get() {
        // If dragging
        if !buttons.pressed(MouseButton::Left) {
            is_drag.set(None);
            eprintln!("Released from drag");
            released = true;
            // Stop displaying squares to move
            writer.write(PieceRelease);
        }
        let mut piece_transform = None;
        let mut piece_idx_component = None;
        for (_, idx, transform, _, _) in piece_query.iter_mut() {
            if *idx == original_pos {
                piece_transform = Some(transform);
                piece_idx_component = Some(idx);
            }
        }
        let mut piece_transform =
            piece_transform.expect("Should be dragging existing piece on the board");
        let piece_idx_component =
            piece_idx_component.expect("Should be dragging existing piece on the board");
        if released {
            eprintln!("Released drag");
            let target_index = cords_to_index(cursor_cords.get());
            if let Some(target_pos) = target_index {
                if movable_sqs
                    .get()
                    .expect("Should have movable sqs if just dragging piece")
                    .contains(&target_pos)
                {
                    // If cursor drag is released over a movable square
                    // gui should make move first and also update idx component
                    let legal_move =
                        LegalMove::new_unchecked(*piece_idx_component, target_pos, None);
                    if board.is_promotion_move(legal_move.deref()) {
                        // Display pieces then pause game until promotion type chosen
                        eprintln!("gui: promotion move detected");
                        let mut batches = Vec::new();
                        for (piece_kind, transform) in [
                            (PieceKind::Knight, KNIGHT_PROMOTION_POS),
                            (PieceKind::Bishop, BISHOP_PROMOTION_POS),
                            (PieceKind::Rook, ROOK_PROMOTION_POS),
                            (PieceKind::Queen, QUEEN_PROMOTION_POS),
                        ] {
                            let piece = Piece::new(piece_kind, board.get_turn());
                            let path = piece.to_string() + ".png";
                            batches.push((
                                Sprite::from_image(asset_server.load(path)),
                                Transform::from_translation(transform)
                                    .with_scale(Vec3::splat(0.20)),
                                piece,
                                PromotionPieceMarker,
                            ));
                        }
                        commands.spawn_batch(batches);
                        promotion_type.set(Some(legal_move));
                        return;
                    };
                    gui_make_legal_move(
                        board.deref(),
                        piece_query,
                        legal_move,
                        board_sq_pos.deref(),
                        &mut commands,
                        asset_server,
                    );
                    *board = board.make_legal_move(legal_move);
                } else {
                    // don't make move and snap piece back to original square
                    gui_change_piece_transform(
                        piece_transform.deref_mut(),
                        board_sq_pos[original_pos],
                    );
                }
            } else {
                // move back to original square
                gui_move_piece(
                    piece_transform,
                    piece_idx_component,
                    original_pos,
                    board_sq_pos.deref(),
                );
            }
        } else {
            // Drag to current cursor cords
            gui_change_piece_transform(piece_transform.deref_mut(), cursor_cords.deref().get());
        }
    }
}

fn gui_change_piece_transform(transform: &mut Transform, target_vec: Vec3) {
    *transform = transform.with_translation(target_vec.with_z(1.0));
}

fn gui_move_piece(
    mut transform: Mut<'_, Transform>,
    mut idx_component: Mut<'_, BoardPos>,
    target_pos: BoardPos,
    board_sq_pos: &BoardSqPos,
) {
    *transform = transform.with_translation(board_sq_pos[target_pos]);
    *idx_component = target_pos;
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
    paused: Res<PromotingMove>,
) {
    let mut toggle_highlight_squares = |sqs: &Vec<BoardPos>| {
        //eprintln!("toggling squares: {sqs:?}");
        for (mut transform, idx) in highlight_query.iter_mut() {
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
        press_reader.clear();
    }
    if !release_reader.is_empty() {
        eprintln!(
            "received release event, number of release events {:?}",
            release_reader.len()
        );
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

pub fn promotion_move_system(
    mut board: ResMut<Board>,
    buttons: Res<ButtonInput<MouseButton>>,
    cursor_cords: Res<WorldCursorCoords>,
    board_sq_pos: Res<BoardSqPos>,
    mut promotion_query: Query<Entity, With<PromotionPieceMarker>>,
    piece_query: Query<
        (
            Entity,
            &mut BoardPos,
            &mut Transform,
            &mut Piece,
            &mut Sprite,
        ),
        (With<BoardPos>, Without<HighlightSquare>),
    >,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut promoting_move: ResMut<PromotingMove>,
) {
    if let Some(mv) = promoting_move.deref().get() {
        // not waiting for promotion
        if buttons.just_pressed(MouseButton::Left) {
            let mut distances: Vec<(i32, PieceKind)> = [
                (KNIGHT_PROMOTION_POS, PieceKind::Knight),
                (BISHOP_PROMOTION_POS, PieceKind::Bishop),
                (ROOK_PROMOTION_POS, PieceKind::Rook),
                (QUEEN_PROMOTION_POS, PieceKind::Queen),
            ]
            .iter()
            .map(|(vec, piece_kind)| (distance_from_pos(vec, &cursor_cords.get()), *piece_kind))
            .collect();
            distances.sort_by(|(distance1, _), (distance2, _)| distance1.cmp(distance2));
            eprintln!("distances: {distances:?}");
            let (shortest_distance, closest_piece_kind) =
                distances.first().expect("Should have 4 elements");
            let promotion_type: PieceKind = if *shortest_distance > 100 {
                eprintln!("Shortest distance too far, auto queening");
                PieceKind::Queen
            } else {
                *closest_piece_kind
            };
            let mv = mv.with_promotion_type(promotion_type);
            gui_make_legal_move(
                board.deref(),
                piece_query,
                mv,
                board_sq_pos.deref(),
                &mut commands,
                asset_server,
            );
            *board = board.make_legal_move(mv);
            promoting_move.set(None);
            // despawn promotion options
            for entity in promotion_query.iter_mut() {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn distance_from_pos(pos1: &Vec3, pos2: &Vec3) -> i32 {
    sqrt(powf(pos1.x - pos2.x, 2.0) + powf(pos1.y - pos2.y, 2.0)).trunc() as i32
}
