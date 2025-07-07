use std::{
    fmt::Display,
    ops::{Deref, Index, IndexMut},
    u8,
};

use bevy::ecs::{component::Component, resource::Resource};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Colour {
    Black,
    White,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum BoardDirection {
    Vertical,
    Horizontal,
    DiagonalNE,
    DiagonalNW,
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

    pub fn get_row(&self) -> u8 {
        self.0 / 8
    }

    pub fn get_column(&self) -> u8 {
        self.0 % 8
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CastleRights {
    black: bool,
    white: bool,
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
        }
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
        eprintln!("vec has length (should be 0): {:?}", vec.len());
        let piece_kind = self[idx]
            .get_piece()
            .expect("Cannot call board::movable_squares on an empty square")
            .get_kind();

        match piece_kind {
            PieceKind::Rook => {
                for shift in [
                    BoardPos::try_move_up,
                    BoardPos::try_move_right,
                    BoardPos::try_move_down,
                    BoardPos::try_move_left,
                ] {
                    repeated_search_movable_squares(&shift, &mut vec);
                }
            }
            PieceKind::Bishop => {
                for shift in [
                    BoardPos::try_move_diag_north_east,
                    BoardPos::try_move_diag_north_west,
                    BoardPos::try_move_diag_south_east,
                    BoardPos::try_move_diag_south_west,
                ] {
                    repeated_search_movable_squares(&shift, &mut vec);
                }
            }
            PieceKind::Queen => {
                for shift in [
                    BoardPos::try_move_up,
                    BoardPos::try_move_right,
                    BoardPos::try_move_down,
                    BoardPos::try_move_left,
                    BoardPos::try_move_diag_north_east,
                    BoardPos::try_move_diag_north_west,
                    BoardPos::try_move_diag_south_east,
                    BoardPos::try_move_diag_south_west,
                ] {
                    repeated_search_movable_squares(&shift, &mut vec);
                }
            }
            PieceKind::King => {
                // Check movements
                for shift in [
                    BoardPos::try_move_diag_north_east,
                    BoardPos::try_move_diag_north_west,
                    BoardPos::try_move_diag_south_east,
                    BoardPos::try_move_diag_south_west,
                    BoardPos::try_move_up,
                    BoardPos::try_move_right,
                    BoardPos::try_move_down,
                    BoardPos::try_move_left,
                ] {
                    search_movable_squares(&shift, &mut vec)
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
                for shift in [
                    BoardPos::chain_try_moves(vec![
                        BoardPos::try_move_diag_south_west,
                        BoardPos::try_move_down,
                    ]),
                    BoardPos::chain_try_moves(vec![
                        BoardPos::try_move_diag_south_west,
                        BoardPos::try_move_left,
                    ]),
                    BoardPos::chain_try_moves(vec![
                        BoardPos::try_move_diag_south_east,
                        BoardPos::try_move_down,
                    ]),
                    BoardPos::chain_try_moves(vec![
                        BoardPos::try_move_diag_south_east,
                        BoardPos::try_move_right,
                    ]),
                    BoardPos::chain_try_moves(vec![
                        BoardPos::try_move_diag_north_east,
                        BoardPos::try_move_up,
                    ]),
                    BoardPos::chain_try_moves(vec![
                        BoardPos::try_move_diag_north_east,
                        BoardPos::try_move_right,
                    ]),
                    BoardPos::chain_try_moves(vec![
                        BoardPos::try_move_diag_north_west,
                        BoardPos::try_move_up,
                    ]),
                    BoardPos::chain_try_moves(vec![
                        BoardPos::try_move_diag_north_west,
                        BoardPos::try_move_left,
                    ]),
                ] {
                    search_movable_squares(&shift, &mut vec);
                }
            }
            PieceKind::Pawn => {
                for shift in [
                    BoardPos::try_move_up,
                    BoardPos::try_move_diag_north_east,
                    BoardPos::try_move_diag_north_west,
                ] {
                    search_movable_squares(&shift, &mut vec);
                }
            }
        }
        vec
    }

    pub fn legal_movable_squares(&self, idx: BoardPos) -> Vec<BoardPos> {
        if self.in_check {
            // Find attacking moves
            Vec::new()
        } else {
            self.pseudolegal_movable_squares(idx)
        }
    }

    fn is_move_out_of_check(&self, mv: Move) -> bool {
        if self.in_check {
            let last_mv = self
                .last_move
                .expect("Shouldn't call is_move_out_of_check when last move doesn't exist")
                .0;
        } else {
            panic!("calling is_move_out_of_check when not in check");
        }
        false
    }

    fn is_piece_absolute_pinned(&self, idx: BoardPos, direction: BoardDirection) -> bool {
        let move_fn = match direction {
            BoardDirection::Vertical => BoardPos::try_move_up,
            BoardDirection::Horizontal => BoardPos::try_move_right,
            BoardDirection::DiagonalNE => BoardPos::try_move_diag_north_east,
            BoardDirection::DiagonalNW => BoardPos::try_move_diag_north_west,
        };

        let opposite_move_fn = match direction {
            BoardDirection::Vertical => BoardPos::try_move_down,
            BoardDirection::Horizontal => BoardPos::try_move_left,
            BoardDirection::DiagonalNE => BoardPos::try_move_diag_south_west,
            BoardDirection::DiagonalNW => BoardPos::try_move_diag_south_east,
        };
        let piece = self[idx]
            .get_piece()
            .expect("Expected piece found empty square");
        eprintln!("pin fn received piece: {piece:?}");
        let mut king_open_ray_piece: bool = false;
        let mut opponent_open_ray_piece: bool = false;
        let mut search_idx: BoardPos = idx;
        while let Some(new_idx) = move_fn(&search_idx) {
            eprintln!("searched square: {:?}", self[new_idx]);
            match self[new_idx] {
                Square(Some(Piece { kind, colour }))
                    if colour == piece.colour && kind != PieceKind::King =>
                {
                    eprintln!("returning false because found {colour:?} {kind:?}");
                    return false;
                }
                Square(Some(Piece { kind, colour }))
                    if colour == piece.colour && kind == PieceKind::King =>
                {
                    king_open_ray_piece = true;
                    break;
                }
                Square(Some(Piece { kind, colour })) if colour != piece.colour => {
                    if ((direction == BoardDirection::Horizontal
                        || direction == BoardDirection::Vertical)
                        && (kind == PieceKind::Rook || kind == PieceKind::Queen))
                        || ((direction == BoardDirection::DiagonalNE
                            || direction == BoardDirection::DiagonalNW)
                            && (kind == PieceKind::Bishop || kind == PieceKind::Queen))
                    {
                        opponent_open_ray_piece = true;
                        break;
                    } else {
                        eprintln!("returning false because found {colour:?} {kind:?}");
                        return false;
                    }
                }
                _ => {}
            };
            search_idx = new_idx;
        }
        //eprintln!("finished forward search");
        search_idx = idx;
        while let Some(new_idx) = opposite_move_fn(&search_idx) {
            eprintln!("searched square: {:?}", self[new_idx]);
            match self[new_idx] {
                Square(Some(Piece { kind, colour }))
                    if colour == piece.colour && kind != PieceKind::King =>
                {
                    //eprintln!("returning false because found {colour:?} {kind:?}");
                    return false;
                }
                Square(Some(Piece { kind, colour }))
                    if colour == piece.colour && kind == PieceKind::King =>
                {
                    king_open_ray_piece = true;
                    break;
                }
                Square(Some(Piece { kind, colour })) if colour != piece.colour => {
                    if ((direction == BoardDirection::Horizontal
                        || direction == BoardDirection::Vertical)
                        && (kind == PieceKind::Rook || kind == PieceKind::Queen))
                        || ((direction == BoardDirection::DiagonalNE
                            || direction == BoardDirection::DiagonalNW)
                            && (kind == PieceKind::Bishop || kind == PieceKind::Queen))
                    {
                        opponent_open_ray_piece = true;
                        break;
                    } else {
                        //eprintln!("returning false because found {colour:?} {kind:?}");
                        return false;
                    }
                }
                _ => {}
            };
            search_idx = new_idx;
        }
        //eprintln!("pin fn opponent_open_ray_piece: {opponent_open_ray_piece:?}, king_open_ray_piece: {king_open_ray_piece:?}");
        opponent_open_ray_piece && king_open_ray_piece
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
        assert_eq!(moves, vec![BoardPos(16), BoardPos(17)]);
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
}
