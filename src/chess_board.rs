use std::ops::{Index, IndexMut};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BoardPos(u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Board([Square; 64]);

#[derive(Debug, Clone, Copy)]
pub struct Move {
    from: BoardPos,
    to: BoardPos,
    promotion_type: Option<PieceKind>,
}

#[derive(Debug, Clone, Copy)]
pub struct LegalMove(pub Move);

enum MoveError {
    OutOfBound,
    PieceNotExists,
}

impl Index<u8> for Board {
    type Output = Square;
    fn index(&self, index: u8) -> &Self::Output {
        &self.0[index as usize]
    }
}
impl IndexMut<u8> for Board {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        &mut self.0[index as usize]
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
        Board(board)
    }

    pub fn make_legal_move(mut self, mv: LegalMove) -> Self {
        let from = mv.0.from.0;
        let to = mv.0.to.0;
        if let Square(Some(piece)) = self[from] {
            use Colour::*;
            use PieceKind::*;
            match piece {
                Piece {
                    kind: King,
                    colour: White,
                } => {
                    // Check for short castle white
                    if to == 6 && from == 4 {
                        // Move rook
                        self.0[5] = sq!(Rook, White);
                        self.0[7] = sq!();
                    } else if to == 2 && from == 4 {
                        // long castle white
                        // Move rook
                        self.0[3] = sq!(Rook, White);
                        self.0[0] = sq!();
                    }
                }
                Piece {
                    kind: King,
                    colour: Black,
                } => {
                    // Check for short castle black
                    if to == 6 + 56 && from == 60 {
                        // Move rook
                        self.0[5 + 56] = sq!(Rook, Black);
                        self.0[7 + 56] = sq!();
                    } else if to == 58 && from == 60 {
                        // long castle black
                        // Move rook
                        self.0[3 + 56] = sq!(Rook, Black);
                        self.0[56] = sq!();
                    }
                }
                Piece { kind: Pawn, colour } => {
                    let from_file = from % 8;
                    let to_file = to % 8;
                    // En passant: diagonal move but target square is empty
                    if (to_file != from_file) && self[to].0.is_none() {
                        let captured_pawn_pos = match colour {
                            Black => to - 8,
                            White => to + 8,
                        };
                        self[captured_pawn_pos] = sq!();
                    } else if (to_file, colour) == (7, White) || (to_file, colour) == (0, Black) {
                    }
                }
                _ => {}
            }
            self[to] = Square(Some(piece));
            self[from] = sq!();
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_board() {
        let board = Board::create_standard();
        assert!(matches!(
            board[0].0,
            Some(Piece {
                kind: PieceKind::Rook,
                colour: Colour::White
            })
        ));
        assert!(matches!(
            board[63].0,
            Some(Piece {
                kind: PieceKind::Rook,
                colour: Colour::Black
            })
        ));
        assert!(board[4].0.unwrap().kind == PieceKind::King);
        assert!(matches!(
            board[1].0,
            Some(Piece {
                kind: PieceKind::Knight,
                colour: Colour::White,
            }),
        ));
    }
}
