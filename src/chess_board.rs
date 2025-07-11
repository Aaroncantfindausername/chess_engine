use std::{
    fmt::Display,
    ops::{Deref, Index, IndexMut},
    panic::panic_any,
    u8,
};

use bevy::{
    ecs::{component::Component, resource::Resource},
    state::state::last_transition,
};

macro_rules! sq {
    ($kind:ident, $colour:ident) => {
        Square(Some(Piece {
            kind: PieceKind::$kind,
            colour: Colour::$colour,
        }))
    };
    () => {
        Square(None)
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceKind {
    King,
    Rook,
    Pawn,
    Queen,
    Bishop,
    Knight,
}

impl PieceKind {
    fn get_move_functions(&self) -> Vec<Box<dyn Fn(&BoardPos) -> Option<BoardPos>>> {
        match *self {
            Self::Knight => vec![
                Box::new(BoardPos::chain_try_moves(vec![
                    BoardPos::try_move_diag_south_west,
                    BoardPos::try_move_down,
                ])),
                Box::new(BoardPos::chain_try_moves(vec![
                    BoardPos::try_move_diag_south_west,
                    BoardPos::try_move_left,
                ])),
                Box::new(BoardPos::chain_try_moves(vec![
                    BoardPos::try_move_diag_south_east,
                    BoardPos::try_move_down,
                ])),
                Box::new(BoardPos::chain_try_moves(vec![
                    BoardPos::try_move_diag_south_east,
                    BoardPos::try_move_right,
                ])),
                Box::new(BoardPos::chain_try_moves(vec![
                    BoardPos::try_move_diag_north_east,
                    BoardPos::try_move_up,
                ])),
                Box::new(BoardPos::chain_try_moves(vec![
                    BoardPos::try_move_diag_north_east,
                    BoardPos::try_move_right,
                ])),
                Box::new(BoardPos::chain_try_moves(vec![
                    BoardPos::try_move_diag_north_west,
                    BoardPos::try_move_up,
                ])),
                Box::new(BoardPos::chain_try_moves(vec![
                    BoardPos::try_move_diag_north_west,
                    BoardPos::try_move_left,
                ])),
            ],
            Self::King => vec![
                Box::new(BoardPos::try_move_up),
                Box::new(BoardPos::try_move_down),
                Box::new(BoardPos::try_move_left),
                Box::new(BoardPos::try_move_right),
                Box::new(BoardPos::try_move_diag_north_east),
                Box::new(BoardPos::try_move_diag_south_east),
                Box::new(BoardPos::try_move_diag_south_west),
                Box::new(BoardPos::try_move_diag_north_west),
            ],

            Self::Rook => vec![
                Box::new(BoardPos::try_move_up),
                Box::new(BoardPos::try_move_down),
                Box::new(BoardPos::try_move_left),
                Box::new(BoardPos::try_move_right),
            ],

            Self::Queen => vec![
                Box::new(BoardPos::try_move_up),
                Box::new(BoardPos::try_move_down),
                Box::new(BoardPos::try_move_left),
                Box::new(BoardPos::try_move_right),
                Box::new(BoardPos::try_move_diag_north_east),
                Box::new(BoardPos::try_move_diag_south_east),
                Box::new(BoardPos::try_move_diag_south_west),
                Box::new(BoardPos::try_move_diag_north_west),
            ],

            Self::Bishop => vec![
                Box::new(BoardPos::try_move_diag_north_east),
                Box::new(BoardPos::try_move_diag_south_east),
                Box::new(BoardPos::try_move_diag_south_west),
                Box::new(BoardPos::try_move_diag_north_west),
            ],
            Self::Pawn => Vec::new(),
        }
    }

    fn is_ray_piece(&self) -> bool {
        match *self {
            Self::King => false,
            Self::Rook => true,
            Self::Pawn => false,
            Self::Knight => false,
            Self::Queen => true,
            Self::Bishop => true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Colour {
    Black,
    White,
}

impl Colour {
    const WHITE_EN_PASSANT_ROW: u8 = 4;
    const BLACK_EN_PASSANT_ROW: u8 = 3;
    const WHITE_PAWN_ROW: u8 = 1;
    const BLACK_PAWN_ROW: u8 = 6;
    pub fn other_colour(&self) -> Self {
        match *self {
            Colour::Black => Colour::White,
            Colour::White => Colour::Black,
        }
    }
    fn get_en_passant_row(&self) -> u8 {
        match *self {
            Colour::Black => Colour::BLACK_EN_PASSANT_ROW,
            Colour::White => Colour::WHITE_EN_PASSANT_ROW,
        }
    }

    fn get_starting_pawn_row(&self) -> u8 {
        match *self {
            Colour::Black => Colour::BLACK_PAWN_ROW,
            Colour::White => Colour::WHITE_PAWN_ROW,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum BoardDirection {
    Vertical,
    Horizontal,
    DiagonalNE,
    DiagonalNW,
}

impl BoardDirection {
    fn get_move_functions(
        &self,
    ) -> (
        fn(&BoardPos) -> Option<BoardPos>,
        fn(&BoardPos) -> Option<BoardPos>,
    ) {
        match *self {
            Self::Vertical => (BoardPos::try_move_up, BoardPos::try_move_down),
            Self::Horizontal => (BoardPos::try_move_right, BoardPos::try_move_left),
            Self::DiagonalNE => (
                BoardPos::try_move_diag_north_east,
                BoardPos::try_move_diag_south_west,
            ),
            Self::DiagonalNW => (
                BoardPos::try_move_diag_north_west,
                BoardPos::try_move_diag_south_east,
            ),
        }
    }

    fn relative_directions(idx1: BoardPos, idx2: BoardPos) -> Option<BoardDirection> {
        if idx1.get_row() == idx2.get_row() {
            Some(BoardDirection::Horizontal)
        } else if idx1.get_column() == idx2.get_column() {
            Some(BoardDirection::Vertical)
        } else if idx1.is_in_same_north_east_diagonal(idx2) {
            Some(BoardDirection::DiagonalNE)
        } else if idx1.is_in_same_north_west_diagonal(idx2) {
            Some(BoardDirection::DiagonalNW)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ToMove(Colour);
impl Deref for ToMove {
    type Target = Colour;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ToMove {
    pub fn new(colour: Colour) -> Self {
        ToMove(colour)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component, Hash)]
pub struct Piece {
    kind: PieceKind,
    colour: Colour,
}

impl Piece {
    pub fn get_kind(&self) -> PieceKind {
        self.kind
    }
    pub fn get_colour(&self) -> Colour {
        self.colour
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Square(Option<Piece>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Component)]
pub struct BoardPos(u8);

impl BoardPos {
    pub fn new(pos: u8) -> Self {
        BoardPos(pos)
    }

    fn get_row(&self) -> u8 {
        self.0 / 8
    }

    fn get_column(&self) -> u8 {
        self.0 % 8
    }

    fn is_in_same_column_or_row(&self, idx: BoardPos) -> bool {
        self.get_row() == idx.get_row() || self.get_column() == idx.get_column()
    }

    fn is_in_same_diagonal(&self, idx: BoardPos) -> bool {
        let row_diff = self.get_row().abs_diff(idx.get_row());
        let column_diff = self.get_column().abs_diff(idx.get_column());
        column_diff == row_diff
    }

    fn is_in_same_north_east_diagonal(&self, idx: BoardPos) -> bool {
        let row_diff = self.get_row() as isize - idx.get_row() as isize;
        let column_diff = self.get_column() as isize - idx.get_column() as isize;
        column_diff == row_diff
    }
    fn is_in_same_north_west_diagonal(&self, idx: BoardPos) -> bool {
        let row_diff = self.get_row() as isize - idx.get_row() as isize;
        let column_diff = self.get_column() as isize - idx.get_column() as isize;
        column_diff == -row_diff
    }

    pub fn try_move_up(&self) -> Option<BoardPos> {
        if self.get_row() < 7 {
            Some(BoardPos::new(self.0 + 8))
        } else {
            None
        }
    }

    pub fn try_move_down(&self) -> Option<BoardPos> {
        if self.get_row() > 0 {
            Some(BoardPos::new(self.0 - 8))
        } else {
            None
        }
    }

    pub fn try_move_left(&self) -> Option<BoardPos> {
        if self.get_column() > 0 {
            Some(BoardPos::new(self.0 - 1))
        } else {
            None
        }
    }

    pub fn try_move_right(&self) -> Option<BoardPos> {
        if self.get_column() < 7 {
            Some(BoardPos::new(self.0 + 1))
        } else {
            None
        }
    }

    pub fn chain_try_moves(
        moves: Vec<impl Fn(&BoardPos) -> Option<BoardPos>>,
    ) -> impl Fn(&BoardPos) -> Option<BoardPos> {
        move |idx: &BoardPos| moves.iter().try_fold(*idx, |acc, elem| elem(&acc))
    }

    pub fn try_move_diag_north_east(&self) -> Option<BoardPos> {
        BoardPos::chain_try_moves(vec![BoardPos::try_move_up, BoardPos::try_move_right])(self)
    }

    pub fn try_move_diag_north_west(&self) -> Option<BoardPos> {
        BoardPos::chain_try_moves(vec![BoardPos::try_move_up, BoardPos::try_move_left])(self)
    }
    pub fn try_move_diag_south_east(&self) -> Option<BoardPos> {
        BoardPos::chain_try_moves(vec![BoardPos::try_move_down, BoardPos::try_move_right])(self)
    }
    pub fn try_move_diag_south_west(&self) -> Option<BoardPos> {
        BoardPos::chain_try_moves(vec![BoardPos::try_move_down, BoardPos::try_move_left])(self)
    }
}

impl From<BoardPos> for usize {
    fn from(value: BoardPos) -> Self {
        value.0 as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Resource)]
pub struct Board {
    squares: [Square; 64],
    pub turn: ToMove,
    in_check: bool,
    castle_rights: CastleRights,
    last_move: Option<LegalMove>,
    king_pos: KingPositions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CastleRights {
    black: bool,
    white: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct KingPositions {
    black: BoardPos,
    white: BoardPos,
}
impl KingPositions {
    fn get(&self, colour: Colour) -> BoardPos {
        match colour {
            Colour::Black => self.black,
            Colour::White => self.white,
        }
    }
}

impl Default for KingPositions {
    fn default() -> Self {
        KingPositions {
            black: BoardPos(60),
            white: BoardPos(4),
        }
    }
}

impl CastleRights {
    pub fn get(&self, colour: Colour) -> bool {
        match colour {
            Colour::Black => self.black,
            Colour::White => self.white,
        }
    }

    pub fn set(&mut self, colour: Colour, val: bool) {
        match colour {
            Colour::Black => self.black = val,
            Colour::White => self.white = val,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Move {
    from: BoardPos,
    to: BoardPos,
    promotion_type: Option<PieceKind>,
}

impl Move {
    fn new(from: BoardPos, to: BoardPos, promotion_type: Option<PieceKind>) -> Self {
        Move {
            from,
            to,
            promotion_type,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LegalMove(Move);

impl Deref for LegalMove {
    type Target = Move;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

enum MoveError {
    OutOfBound,
    PieceNotExists,
}

impl Square {
    pub fn get_piece(self) -> Option<Piece> {
        if let Square(Some(piece)) = self {
            return Some(piece);
        }
        None
    }
}

impl Default for ToMove {
    fn default() -> Self {
        ToMove(Colour::White)
    }
}

impl Default for Board {
    fn default() -> Self {
        Board::create_standard()
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let piece = match self.kind {
            PieceKind::King => "king",
            PieceKind::Rook => "rook",
            PieceKind::Pawn => "pawn",
            PieceKind::Queen => "queen",
            PieceKind::Bishop => "bishop",
            PieceKind::Knight => "knight",
        };
        let colour = match self.colour {
            Colour::Black => "_black",
            Colour::White => "_white",
        };
        write!(f, "{piece}{colour}")
    }
}

impl Index<BoardPos> for Board {
    type Output = Square;
    fn index(&self, index: BoardPos) -> &Self::Output {
        &self.squares[index.0 as usize]
    }
}
impl IndexMut<BoardPos> for Board {
    fn index_mut(&mut self, index: BoardPos) -> &mut Self::Output {
        &mut self.squares[index.0 as usize]
    }
}

impl BoardPos {
    pub const A1: BoardPos = BoardPos(0);
    pub const B1: BoardPos = BoardPos(1);
    pub const H1: BoardPos = BoardPos(7);
    pub const E1: BoardPos = BoardPos(4);
    pub const F1: BoardPos = BoardPos(5);
    pub const G1: BoardPos = BoardPos(6);
    pub const C1: BoardPos = BoardPos(2);
    pub const D1: BoardPos = BoardPos(3);
    pub const A8: BoardPos = BoardPos(56);
    pub const B8: BoardPos = BoardPos(1 + 56);
    pub const H8: BoardPos = BoardPos(7 + 56);
    pub const E8: BoardPos = BoardPos(4 + 56);
    pub const F8: BoardPos = BoardPos(5 + 56);
    pub const G8: BoardPos = BoardPos(6 + 56);
    pub const C8: BoardPos = BoardPos(2 + 56);
    pub const D8: BoardPos = BoardPos(3 + 56);
}

impl Board {
    pub fn get_turn(&self) -> ToMove {
        self.turn
    }

    pub fn in_check(&self) -> bool {
        self.in_check
    }

    pub fn has_castle_rights(&self) -> bool {
        self.castle_rights.get(self.turn.0)
    }

    fn get_current_turn_king_pos(&self) -> BoardPos {
        self.king_pos.get(self.turn.0)
    }

    pub fn create_standard() -> Self {
        use Colour::*;
        use PieceKind::*;
        let backrank = [Rook, Knight, Bishop, Queen, King, Bishop, Knight, Rook];
        let mut board: [Square; 64] = [Square(None); 64];
        let mut place = |row: usize, column: usize, piece: PieceKind, colour: Colour| {
            board[row * 8 + column] = Square(Some(Piece {
                kind: piece,
                colour,
            }))
        };
        // Place black backrank
        for (column, &piece) in backrank.iter().enumerate() {
            place(0, column, piece, White)
        }
        // Place white backrank
        for (column, &piece) in backrank.iter().enumerate() {
            place(7, column, piece, Black)
        }
        // Place pawns
        for column in 0..8 {
            place(1, column, Pawn, White);
            place(6, column, Pawn, Black);
        }
        println!("{:?}", board[63]);
        Board {
            squares: board,
            turn: ToMove::new(Colour::White),
            in_check: false,
            castle_rights: CastleRights {
                black: true,
                white: true,
            },
            last_move: None,
            king_pos: KingPositions::default(),
        }
    }

    fn can_en_passant(&self, idx: BoardPos) -> bool {
        let pawn = self[idx]
            .get_piece()
            .expect("should be checking en passant for non empty square");
        if pawn.kind != PieceKind::Pawn || pawn.colour != self.turn.0 {
            panic!("should be checking en passant for pawns on the side to move only");
        }
        if let Some(LegalMove(last_move)) = self.last_move {
            if self[last_move.to]
                != Square(Some(Piece {
                    kind: PieceKind::Pawn,
                    colour: pawn.colour.other_colour(),
                }))
            {
                return false;
            } else if idx.get_row() != pawn.colour.get_en_passant_row() {
                return false;
            } else if last_move.to.get_row() != pawn.colour.get_en_passant_row()
                || last_move.from.get_row() != pawn.colour.get_starting_pawn_row()
            {
                return false;
            } else if last_move.to.get_column() >= idx.get_column() {
                if last_move.to.get_column() - idx.get_column() != 1 {
                    return false;
                } else if self[idx
                    .try_move_diag_south_east()
                    .expect("should be able to move diag south east")]
                .0
                .is_some()
                {
                    return false;
                }
            } else if last_move.to.get_column() < idx.get_column() {
                if idx.get_column() - last_move.to.get_column() != 1 {
                    return false;
                } else if self[idx
                    .try_move_diag_south_west()
                    .expect("should be able to move diag south west")]
                .0
                .is_some()
                {
                    return false;
                }
            }
        } else {
            return false;
        }
        true
    }

    fn is_move_en_passant(&self, mv: Move) -> bool {
        if mv.promotion_type.is_some() {
            return false;
        }
        if mv.from.get_column() != mv.to.get_column() {
            // Diagonal move
            if self[mv.to].get_piece().is_none() {
                // but no capture
                return true;
            }
        }
        false
    }

    pub fn make_legal_move(mut self, mv: LegalMove) -> Self {
        let from = mv.0.from;
        let to = mv.0.to;
        if let Square(Some(piece)) = self[from] {
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
                        self[BoardPos::F1] = sq!(Rook, White);
                        self[BoardPos::H1] = sq!();
                    } else if to.0 == 2 && from.0 == 4 {
                        // long castle white
                        // Move rook
                        self[BoardPos::D1] = sq!(Rook, White);
                        self[BoardPos::A1] = sq!();
                    }
                }
                Piece {
                    kind: King,
                    colour: Black,
                } => {
                    // Check for short castle black
                    if to == BoardPos::G8 && from == BoardPos::E8 {
                        // Move rook
                        self[BoardPos::F8] = sq!(Rook, Black);
                        self[BoardPos::H8] = sq!();
                    } else if to == BoardPos::C8 && from == BoardPos::E8 {
                        // long castle black
                        // Move rook
                        self[BoardPos::D8] = sq!(Rook, Black);
                        self[BoardPos::A8] = sq!();
                    }
                    //TODO make sure to update king positions and castle only when not in check
                }
                Piece { kind: Pawn, colour } => {
                    let from_file = from.0 % 8;
                    let to_file = to.0 % 8;
                    // En passant: diagonal move but target square is empty
                    if (to_file != from_file) && self[to].0.is_none() {
                        let captured_pawn_pos = match colour {
                            Black => to.0 - 8,
                            White => to.0 + 8,
                        };
                        self[BoardPos(captured_pawn_pos)] = sq!();
                    } else if (to_file, colour) == (7, White) || (to_file, colour) == (0, Black) {
                        let kind =
                            mv.0.promotion_type
                                .expect("No promotion_type given for move");
                        self[to] = Square(Some(Piece { kind, colour }));
                        self[from] = sq!();
                        return self;
                    }
                }
                _ => {}
            }
            // TODO need to update if in check and castle rights
            self[to] = Square(Some(piece));
            self[from] = sq!();
        }
        self
    }

    fn pseudolegal_movable_squares(&self, idx: BoardPos) -> Vec<BoardPos> {
        let search_movable_squares =
            |shift: &dyn Fn(&BoardPos) -> Option<BoardPos>, vec: &mut Vec<BoardPos>| {
                if let Some(possible_board_pos) = shift(&idx) {
                    if let Some(Piece { kind, colour }) = self[possible_board_pos].get_piece() {
                        if colour != *self.get_turn() {
                            vec.push(possible_board_pos);
                        }
                    } else {
                        vec.push(possible_board_pos);
                    }
                }
            };
        // Closure to continue searching and adding movable squares in a direction according to
        // a function
        let repeated_search_movable_squares =
            |shift: &dyn Fn(&BoardPos) -> Option<BoardPos>, vec: &mut Vec<BoardPos>| {
                while let Some(possible_board_pos) = shift(&idx) {
                    if let Square(Some(piece)) = self[possible_board_pos] {
                        // If moving into squares with ally pieces, stop searching
                        if piece.get_colour() == *self.turn {
                            break;
                        } else {
                            // If moving into square with enemy piece, add that postition then
                            // stop searching
                            vec.push(possible_board_pos);
                            break;
                        }
                    }
                    vec.push(possible_board_pos);
                }
            };
        let mut vec: Vec<BoardPos> = Vec::new();
        let piece_kind = self[idx]
            .get_piece()
            .expect("Cannot call board::movable_squares on an empty square")
            .get_kind();
        let piece_colour = self[idx]
            .get_piece()
            .expect("Cannot call board::movable_squares on an empty square")
            .get_colour();
        let move_fns = piece_kind.get_move_functions();
        match piece_kind {
            PieceKind::Rook => {
                for shift in move_fns {
                    repeated_search_movable_squares(shift.deref(), &mut vec);
                }
            }
            PieceKind::Bishop => {
                for shift in move_fns {
                    repeated_search_movable_squares(shift.deref(), &mut vec);
                }
            }
            PieceKind::Queen => {
                for shift in move_fns {
                    repeated_search_movable_squares(shift.deref(), &mut vec);
                }
            }
            PieceKind::King => {
                // Check movements
                for shift in move_fns {
                    search_movable_squares(shift.deref(), &mut vec)
                }
                // Check if can castle, and if so add the positions
                if self.has_castle_rights() {
                    let mut can_long_castle = true;
                    for idx in [BoardPos::B1, BoardPos::C1, BoardPos::D1] {
                        if self[idx].get_piece().is_some() {
                            can_long_castle = false;
                        }
                    }
                    if can_long_castle {
                        vec.push(BoardPos::C1);
                    }
                    let mut can_short_castle = true;
                    for idx in [BoardPos::F1, BoardPos::G1] {
                        if self[idx].get_piece().is_some() {
                            can_short_castle = false;
                        }
                    }
                    if can_short_castle {
                        vec.push(BoardPos::G1);
                    }
                }
            }
            PieceKind::Knight => {
                for shift in move_fns {
                    search_movable_squares(shift.deref(), &mut vec);
                }
            }
            PieceKind::Pawn => {
                let pawn_search_movable_squares =
                    |vertical_shift: &dyn Fn(&BoardPos) -> Option<BoardPos>,
                     diag_shifts: &[&dyn Fn(&BoardPos) -> Option<BoardPos>; 2],
                     vec: &mut Vec<BoardPos>| {
                        if let Some(possible_board_pos) = vertical_shift(&idx) {
                            if self[possible_board_pos].get_piece().is_none() {
                                vec.push(possible_board_pos);
                            }
                        }
                        for &shift in diag_shifts {
                            if let Some(possible_board_pos) = shift(&idx) {
                                if let Some(Piece { kind, colour }) =
                                    self[possible_board_pos].get_piece()
                                {
                                    if colour != piece_colour {
                                        vec.push(possible_board_pos);
                                    }
                                }
                            }
                        }
                    };
                //Check if starting row
                let mut vert_movement: &dyn Fn(&BoardPos) -> Option<BoardPos>;
                let diagonal_movements: &[&dyn Fn(&BoardPos) -> Option<BoardPos>; 2];
                vert_movement = match piece_colour {
                    Colour::Black => &BoardPos::try_move_down,
                    Colour::White => &BoardPos::try_move_up,
                };
                diagonal_movements = match piece_colour {
                    Colour::Black => &[
                        &BoardPos::try_move_diag_south_east,
                        &BoardPos::try_move_diag_south_west,
                    ],
                    Colour::White => &[
                        &BoardPos::try_move_diag_north_east,
                        &BoardPos::try_move_diag_north_west,
                    ],
                };
                if idx.get_row() == piece_colour.get_starting_pawn_row() {
                    vert_movement = match piece_colour {
                        Colour::Black => &BoardPos::try_move_down,
                        Colour::White => &BoardPos::try_move_up,
                    };
                    let one_forward_pos = vert_movement(&idx)
                        .expect("should be able to move forwards as pawn from starting row");
                    let two_forward_pos = vert_movement(&one_forward_pos).expect(
                        "should be able to move forwards 2 spaces as pawn from starting row",
                    );
                    if self[two_forward_pos].0.is_none() && self[one_forward_pos].0.is_none() {
                        // If starting row pawn and forwand 2 positions are empty, add move forward
                        // 2 squares to possible moves positions
                        vec.push(two_forward_pos);
                    }
                }
                pawn_search_movable_squares(&vert_movement, diagonal_movements, &mut vec);
                if self.can_en_passant(idx) {
                    let last_move_column = self
                        .last_move
                        .expect("should have last move if can en passant")
                        .to
                        .get_column();
                    if idx.get_column() > last_move_column {
                        match piece_colour {
                            Colour::Black => vec.push(
                                idx.try_move_diag_south_east()
                                    .expect("Should be valid since can en passant"),
                            ),
                            Colour::White => vec.push(
                                idx.try_move_diag_north_east()
                                    .expect("Should be valid since can en passant"),
                            ),
                        }
                    } else {
                        match piece_colour {
                            Colour::Black => vec.push(
                                idx.try_move_diag_south_west()
                                    .expect("Should be valid since can en passant"),
                            ),
                            Colour::White => vec.push(
                                idx.try_move_diag_north_west()
                                    .expect("Should be valid since can en passant"),
                            ),
                        }
                    }
                }
            }
        }
        vec
    }

    fn attacking_squares(&self, idx: BoardPos) -> Vec<BoardPos> {
        let piece_colour = self[idx]
            .0
            .expect("should be finding attacking squares for non-empty square")
            .colour;
        let other_colour = piece_colour.other_colour();
        let mut attacking_positions: Vec<BoardPos> = Vec::new();
        let directional_move_fns = PieceKind::Queen.get_move_functions();
        let kinght_move_fns = PieceKind::Knight.get_move_functions();
        for shift in directional_move_fns {
            let mut search_depth: usize = 0;
            let mut try_pos = idx;
            while let Some(possible_board_pos) = shift(&try_pos) {
                search_depth += 1;
                if let Square(Some(searched_piece)) = self[possible_board_pos] {
                    // If moving into squares with ally pieces, stop searching
                    if searched_piece.get_colour() == piece_colour {
                        break;
                    } else {
                        // If find square with enemy piece, add that postition if the piece can
                        // attack the piece then
                        // stop searching in that direction
                        match searched_piece.kind {
                            PieceKind::Rook => {
                                if idx.is_in_same_column_or_row(possible_board_pos) {
                                    attacking_positions.push(possible_board_pos);
                                }
                            }
                            PieceKind::Bishop => {
                                if idx.is_in_same_diagonal(possible_board_pos) {
                                    attacking_positions.push(possible_board_pos);
                                }
                            }
                            PieceKind::Queen => {
                                if idx.is_in_same_column_or_row(possible_board_pos)
                                    || idx.is_in_same_diagonal(possible_board_pos)
                                {
                                    attacking_positions.push(possible_board_pos);
                                }
                            }
                            PieceKind::King => {
                                if (idx.is_in_same_column_or_row(possible_board_pos)
                                    || idx.is_in_same_diagonal(possible_board_pos))
                                    && search_depth == 1
                                {
                                    attacking_positions.push(possible_board_pos);
                                }
                            }
                            PieceKind::Pawn => {
                                if search_depth == 1 && idx.is_in_same_diagonal(possible_board_pos)
                                {
                                    match other_colour {
                                        Colour::Black => {
                                            if idx.get_row() < possible_board_pos.get_row() {
                                                attacking_positions.push(possible_board_pos);
                                            }
                                        }
                                        Colour::White => {
                                            if idx.get_row() > possible_board_pos.get_row() {
                                                attacking_positions.push(possible_board_pos);
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                        break;
                    }
                }
                try_pos = possible_board_pos;
            }
        }
        // now check knight attackable squares
        for shift in kinght_move_fns {
            if let Some(possible_board_pos) = shift(&idx) {
                if let Square(Some(searched_piece)) = self[possible_board_pos] {
                    // If moving into squares with ally pieces, stop searching
                    if searched_piece.get_colour() != piece_colour
                        && searched_piece.kind == PieceKind::Knight
                    {
                        attacking_positions.push(possible_board_pos);
                    }
                }
            }
        }
        attacking_positions
    }

    fn move_leave_in_check(&self, mv: Move) -> bool {
        let colour = self.turn.0;
        let board = self; // make move on test board and see if king under attack
        board.make_legal_move(LegalMove(mv));
        !board
            .attacking_squares(board.king_pos.get(colour))
            .is_empty()
    }

    fn does_pos_block_attack(
        king_pos: BoardPos,
        attacking_pos: BoardPos,
        blocking_pos: BoardPos,
    ) -> bool {
        let attack_direction = BoardDirection::relative_directions(king_pos, attacking_pos)
            .expect("fn assumes that king is being attacked");
        if BoardDirection::relative_directions(king_pos, blocking_pos).is_none() {
            // blocking piece not in same direction as attcking piece
            return false;
        }
        match attack_direction {
            BoardDirection::Vertical => {
                (king_pos.get_row()..attacking_pos.get_row()).contains(&blocking_pos.get_row())
                    || (attacking_pos.get_row()..king_pos.get_row())
                        .contains(&blocking_pos.get_row())
            }
            _ => {
                // for horizontal and all diagonal directions (diagonal directions can check row or
                // column because we already assume that we're in the correct diagonal)
                (king_pos.get_column()..attacking_pos.get_column())
                    .contains(&blocking_pos.get_column())
                    || (attacking_pos.get_column()..king_pos.get_column())
                        .contains(&blocking_pos.get_column())
            }
        }
    }

    pub fn legal_movable_squares(&self, piece_idx_from: BoardPos) -> Vec<BoardPos> {
        // Check if piece to move is possibly pinned
        let full_test_filter = |to: &BoardPos| {
            self.move_leave_in_check(Move {
                from: piece_idx_from,
                to: *to,
                promotion_type: None,
            })
        };
        let piece_to_move = self[piece_idx_from]
            .get_piece()
            .expect("Should be considering legal movable squares of non empty squares only");
        let pseudolegal_moveable_squares = self.pseudolegal_movable_squares(piece_idx_from);
        if piece_to_move.kind == PieceKind::King {
            return pseudolegal_moveable_squares
                .into_iter()
                .filter(full_test_filter)
                .collect::<Vec<BoardPos>>();
        }
        // Now we can safely assume piece to move is not king
        if let Some(direction) =
            BoardDirection::relative_directions(piece_idx_from, self.get_current_turn_king_pos())
        {
            if self.is_piece_absolute_pinned(piece_idx_from, direction) {
                // Only moves in pinned direction allowed, i.e cannot move out of absolute pin
                return pseudolegal_moveable_squares
                    .into_iter()
                    .filter(|to: &BoardPos| {
                        BoardDirection::relative_directions(piece_idx_from, *to).is_some_and(
                            |move_direction: BoardDirection| move_direction == direction,
                        )
                    })
                    .collect::<Vec<BoardPos>>();
            }
        }
        // Now can assume piece is not absolutely pinned
        if self.in_check {
            // Find attacking squares
            let mut attacking_squares = self.attacking_squares(self.king_pos.get(self.turn.0));
            if attacking_squares.len() == 2 {
                // double attack on king, only king moves are valid, but since piece_to_move is not
                // king, we can just return empty vec
                Vec::new()
            } else if attacking_squares.len() == 1 {
                let attacking_sq = attacking_squares.pop().expect("vec should have 1 element");
                let attacking_piece_kind = self[attacking_sq]
                    .get_piece()
                    .expect("Should be attacked by a non empty piece")
                    .kind;
                match attacking_piece_kind {
                    PieceKind::Knight | PieceKind::Pawn => {
                        pseudolegal_moveable_squares
                            .into_iter()
                            .filter(|to: &BoardPos| *to == attacking_sq)
                            .collect::<Vec<BoardPos>>() // must capture knight
                    }
                    PieceKind::King => {
                        panic!("King being attacked by another king");
                    }
                    kind => {
                        // Attacked by a rook, queen, or bishop, find attacking direction and
                        // filter by moves to a position in between the king and attacking piece in that direction
                        // or pieces that capture the piece
                        pseudolegal_moveable_squares
                            .into_iter()
                            .filter(|to: &BoardPos| {
                                *to == attacking_sq
                                    || Board::does_pos_block_attack(
                                        self.get_current_turn_king_pos(),
                                        attacking_sq,
                                        *to,
                                    )
                            })
                            .collect::<Vec<BoardPos>>() // must capture knight
                    }
                }
            } else {
                panic!("should not be able to have 3 pieces attacking king");
            }
        } else {
            // check not pinned and castle not leave king in check
            if piece_to_move.kind == PieceKind::Pawn {
                pseudolegal_moveable_squares
                    .into_iter()
                    .filter(full_test_filter)
                    .collect::<Vec<BoardPos>>()
            } else {
                pseudolegal_moveable_squares
            }
        }
    }

    fn is_piece_absolute_pinned(&self, idx: BoardPos, direction: BoardDirection) -> bool {
        let piece = self[idx]
            .get_piece()
            .expect("Expected piece found empty square");
        eprintln!("pin fn received piece: {piece:?}");
        fn scan_ray(
            board: &Board,
            start: BoardPos,
            next_fn: impl Fn(&BoardPos) -> Option<BoardPos>,
            our_colour: Colour,
            expected_threats: &[PieceKind],
        ) -> Option<&'static str> {
            let mut pos = start;
            while let Some(new_idx) = next_fn(&pos) {
                //eprintln!("searched square: {:?}", self[new_idx]);
                match board[new_idx] {
                    Square(Some(Piece { kind, colour })) if colour == our_colour => {
                        if kind == PieceKind::King {
                            return Some("king");
                        } else {
                            return None;
                        }
                    }
                    Square(Some(Piece { kind, colour })) if colour != our_colour => {
                        if expected_threats.contains(&kind) {
                            return Some("threat");
                        } else {
                            return None;
                        }
                    }
                    _ => {}
                };
                pos = new_idx;
            }
            None
        }
        let expected_threats = match direction {
            BoardDirection::Vertical | BoardDirection::Horizontal => {
                &[PieceKind::Rook, PieceKind::Queen]
            }
            BoardDirection::DiagonalNE | BoardDirection::DiagonalNW => {
                &[PieceKind::Bishop, PieceKind::Queen]
            }
        };
        let (forward, backward) = direction.get_move_functions();
        let forward_scan = scan_ray(self, idx, forward, piece.colour, expected_threats);
        let backward_scan = scan_ray(self, idx, backward, piece.colour, expected_threats);
        matches!(
            (forward_scan, backward_scan),
            (Some("king"), Some("threat")) | (Some("threat"), Some("king"))
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_board() {
        let board = Board::create_standard();
        assert!(matches!(
            board[BoardPos(0)].0,
            Some(Piece {
                kind: PieceKind::Rook,
                colour: Colour::White
            })
        ));
        assert!(matches!(
            board[BoardPos(63)].0,
            Some(Piece {
                kind: PieceKind::Rook,
                colour: Colour::Black
            })
        ));
        assert!(board[BoardPos(4)].0.unwrap().kind == PieceKind::King);
        assert!(matches!(
            board[BoardPos(1)].0,
            Some(Piece {
                kind: PieceKind::Knight,
                colour: Colour::White,
            }),
        ));
    }

    #[test]
    fn test_chain_moves() {
        let pos = BoardPos::new(11);
        let x =
            BoardPos::chain_try_moves(vec![BoardPos::try_move_up, BoardPos::try_move_right])(&pos);
        assert_eq!(x, Some(BoardPos::new(20)));
    }

    #[test]
    fn test_possible_moves_pawn() {
        let board = Board::create_standard();
        let moves = board.pseudolegal_movable_squares(BoardPos::new(8));
        assert_eq!(moves, vec![BoardPos(24), BoardPos(16)]);
    }

    #[test]
    fn test_possible_moves_king() {
        let board = Board::create_standard();
        let moves = board.pseudolegal_movable_squares(BoardPos::new(4));
        assert_eq!(moves, Vec::new());
    }

    #[test]
    fn test_absolute_pin() {
        let mut board = Board::create_standard();
        board.squares[52] = Square(Some(Piece {
            kind: PieceKind::Rook,
            colour: Colour::Black,
        }));
        assert!(board.is_piece_absolute_pinned(BoardPos::new(12), BoardDirection::Vertical));
    }
    #[test]
    fn test_attacking_squares() {
        let mut board = Board::create_standard();
        board.squares[52] = sq!(Queen, Black);
        board.squares[12] = sq!();
        board.squares[11] = sq!();
        board.squares[32] = sq!(Bishop, Black);
        let attacking = board.attacking_squares(BoardPos(4));
        assert_eq!(attacking, vec![BoardPos(52), BoardPos(32)]);
    }

    #[test]
    fn test_block_attack() {
        let mut board = Board::create_standard();
        board.squares[12] = sq!();
        board.squares[52] = sq!(Queen, Black);
        let does_block = Board::does_pos_block_attack(
            board.get_current_turn_king_pos(),
            BoardPos(52),
            BoardPos(20),
        );
        let does_not_block = Board::does_pos_block_attack(
            board.get_current_turn_king_pos(),
            BoardPos(52),
            BoardPos(21),
        );
        assert!(does_block);
        assert!(!does_not_block);
    }
}
