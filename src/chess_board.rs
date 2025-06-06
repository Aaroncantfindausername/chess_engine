use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PieceKind {
    King,
    Rook,
    Pawn,
    Queen,
    Bishop,
    Knight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Colour {
    Black,
    White,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Piece {
    kind: PieceKind,
    colour: Colour,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Square(Option<Piece>);

struct MoveNum(i8);
struct BoardPos(u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Board([Square; 64]);

struct Move {
    piece: Piece,
    move_num: MoveNum,
    pos: BoardPos,
}

impl Index<usize> for Board {
    type Output = Square;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
impl IndexMut<usize> for Board {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}
impl Board {
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
            place(0, column, piece, Black)
        }
        // Place white backrank
        for (column, &piece) in backrank.iter().enumerate() {
            place(7, column, piece, White)
        }
        // Place pawns
        for column in 0..8 {
            place(1, column, Pawn, Black);
            place(6, column, Pawn, White);
        }
        Board(board)
    }
}
