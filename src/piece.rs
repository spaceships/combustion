use std::fmt;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn other(&self) -> Color {
        match *self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash)]
pub enum PieceType {
    Pawn,
    Bishop,
    Knight,
    Rook,
    Queen,
    King,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Piece {
    pub kind: PieceType,
    pub color: Color,
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Color::White => write!(f, "white"),
            Color::Black => write!(f, "black"),
        }
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Piece { kind: PieceType::Pawn,   color: Color::White } => write!(f, "P"),
            Piece { kind: PieceType::Bishop, color: Color::White } => write!(f, "B"),
            Piece { kind: PieceType::Knight, color: Color::White } => write!(f, "N"),
            Piece { kind: PieceType::Rook,   color: Color::White } => write!(f, "R"),
            Piece { kind: PieceType::Queen,  color: Color::White } => write!(f, "Q"),
            Piece { kind: PieceType::King,   color: Color::White } => write!(f, "K"),
            Piece { kind: PieceType::Pawn,   color: Color::Black } => write!(f, "p"),
            Piece { kind: PieceType::Bishop, color: Color::Black } => write!(f, "b"),
            Piece { kind: PieceType::Knight, color: Color::Black } => write!(f, "n"),
            Piece { kind: PieceType::Rook,   color: Color::Black } => write!(f, "r"),
            Piece { kind: PieceType::Queen,  color: Color::Black } => write!(f, "q"),
            Piece { kind: PieceType::King,   color: Color::Black } => write!(f, "k"),
        }
    }
}
