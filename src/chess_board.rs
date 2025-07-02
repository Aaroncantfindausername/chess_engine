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

    pub fn try_increment_row(&self) -> Option<u8> {
        let incr_row = (self.0 / 8) + 1;
        if incr_row < 8 {
            Some(incr_row)
        } else {
            None
        }
    }

    pub fn try_decrement_row(&self) -> Option<u8> {
        let decr_row = (self.0 / 8) - 1;
        if decr_row >= 0 {
            Some(decr_row)
        } else {
            None
        }
    }

    pub fn try_increment_column(&self) -> Option<u8> {
        let incr_column = (self.0 % 8) + 1;
        if incr_column < 8 {
            Some(incr_column)
        } else {
            None
        }
    }

    pub fn try_decrement_column(&self) -> Option<u8> {
        let decr_column = (self.0 % 8) - 1;
        if decr_column >= 0 {
            Some(decr_column)
        } else {
            None
        }
    }

    pub fn try_move_diagonally_north_east(&self) -> Option<u8> {
        let new_idx = BoardPos::try_increment_row(&BoardPos(self.try_increment_column()?));
        new_idx
    }

    pub fn try_move_diagonally_north_west(&self) -> Option<u8> {
        let new_idx = BoardPos::try_increment_row(&BoardPos(self.try_decrement_column()?));
        new_idx
    }

    pub fn try_move_diagonally_south_east(&self) -> Option<u8> {
        let new_idx = BoardPos::try_decrement_row(&BoardPos(self.try_increment_column()?));
        new_idx
    }

    pub fn try_move_diagonally_south_west(&self) -> Option<u8> {
        let new_idx = BoardPos::try_decrement_row(&BoardPos(self.try_decrement_column()?));
        new_idx
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
    turn: ToMove,
    in_check: bool,
    castle_rights: CastleRights,
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
    pub const A1: BoardPos = BoardPos(1);
    pub const H1: BoardPos = BoardPos(7);
    pub const E1: BoardPos = BoardPos(4);
    pub const F1: BoardPos = BoardPos(5);
    pub const G1: BoardPos = BoardPos(6);
    pub const C1: BoardPos = BoardPos(2);
    pub const D1: BoardPos = BoardPos(3);
    pub const A8: BoardPos = BoardPos(1 + 56);
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

    pub fn movable_squares(&self, idx: BoardPos) -> Vec<BoardPos> {
        // Closure to continue searching and addind movable squares in a direction according to
        // a function
        let search_movable_squares =
            |shift: fn(&BoardPos) -> Option<u8>, vec: &mut Vec<BoardPos>| {
                while let Some(possible_index) = shift(&idx) {
                    let possible_board_pos = BoardPos::new(possible_index);
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
        eprintln!("vec has lenth: {:?}", vec.len());
        let piece_kind = self[idx]
            .get_piece()
            .expect("Cannot call board::movable_squares on an empty square")
            .get_kind();
        if !self.in_check {
            match piece_kind {
                PieceKind::Rook => {
                    for shift in [
                        BoardPos::try_increment_row,
                        BoardPos::try_increment_column,
                        BoardPos::try_decrement_row,
                        BoardPos::try_decrement_column,
                    ] {
                        search_movable_squares(shift, &mut vec);
                    }
                }
                PieceKind::Bishop => {
                    for shift in [
                        BoardPos::try_move_diagonally_north_east,
                        BoardPos::try_move_diagonally_north_west,
                        BoardPos::try_move_diagonally_south_east,
                        BoardPos::try_move_diagonally_south_west,
                    ] {
                        search_movable_squares(shift, &mut vec);
                    }
                }
                PieceKind::Queen => {
                    for shift in [
                        BoardPos::try_move_diagonally_north_east,
                        BoardPos::try_move_diagonally_north_west,
                        BoardPos::try_move_diagonally_south_east,
                        BoardPos::try_move_diagonally_south_west,
                        BoardPos::try_increment_row,
                        BoardPos::try_increment_column,
                        BoardPos::try_decrement_row,
                        BoardPos::try_decrement_column,
                    ] {
                        search_movable_squares(shift, &mut vec);
                    }
                }
            }
        } else {
            todo!("moves while in check");
        }
        vec
    }

    pub fn legal_moves(&self, idx: BoardPos) -> Vec<LegalMove> {
        Vec::new()
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
}
