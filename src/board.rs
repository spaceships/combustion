use std::fmt;


#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
enum Piece {
    WhitePawn,
    WhiteBishop,
    WhiteKnight,
    WhiteRook,
    WhiteQueen,
    WhiteKing,
    BlackPawn,
    BlackBishop,
    BlackKnight,
    BlackRook,
    BlackQueen,
    BlackKing,
}

pub struct Board {
    board: [Option<Piece>; 64],
    white_turn: bool,
    castle_rights: [bool; 4], // [ white K, white Q, black k, black q ]
    en_passant_target: Option<usize>,
    halfmove_clock: usize,
    move_number: usize,
}

impl Board {
    pub fn new() -> Self {
        Board {
            board: [None; 64],
            white_turn: true,
            castle_rights: [true, true, true, true],
            en_passant_target: None,
            halfmove_clock: 0,
            move_number: 1,
        }
    }

    // "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    pub fn from_fen(fen: &str) -> Self {
        let mut b = Board::new();
        let mut i = 7;
        let mut j = 0;
        for c in fen.chars() {
            match c {
                'r' => {
                    b.board[i*8+j] = Some(Piece::BlackRook);
                    j += 1;
                }

                'R' => {
                    b.board[i*8+j] = Some(Piece::WhiteRook);
                    j += 1;
                }

                _   => {},
            }
        }
        b
    }

}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Piece::WhitePawn   => write!(f, "P"),
            Piece::WhiteBishop => write!(f, "B"),
            Piece::WhiteKnight => write!(f, "N"),
            Piece::WhiteRook   => write!(f, "R"),
            Piece::WhiteQueen  => write!(f, "Q"),
            Piece::WhiteKing   => write!(f, "K"),
            Piece::BlackPawn   => write!(f, "p"),
            Piece::BlackBishop => write!(f, "b"),
            Piece::BlackKnight => write!(f, "n"),
            Piece::BlackRook   => write!(f, "r"),
            Piece::BlackQueen  => write!(f, "q"),
            Piece::BlackKing   => write!(f, "k"),
        }
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..8 {
            write!(f, "[ ")?;
            for j in 0..8 {
                match self.board[i*8+j] {
                    Some(x) => write!(f, "{} ", x)?,
                    None => write!(f, "  ")?,
                }
            }
            write!(f, "]\n")?;
        }
        write!(f, "")
    }
}
