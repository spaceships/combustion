use crate::piece::{Color, Piece, PieceType};
use crate::position::Pos;

use std::fmt;

pub struct Board {
    pub board: [Option<Piece>; 64],
    pub color_to_move: Color,
    pub castle_rights: [bool; 4], // [ white K, white Q, black k, black q ]
    pub en_passant_target: Option<Pos>,
    pub halfmove_clock: usize,
    pub move_number: usize,
}

impl Board {
    pub fn new() -> Self {
        Board {
            board: [None; 64],
            color_to_move: Color::White,
            castle_rights: [false, false, false, false],
            en_passant_target: None,
            halfmove_clock: 0,
            move_number: 1,
        }
    }

    pub fn initial() -> Self {
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }

    pub fn piece(&self, loc: Pos) -> Option<Piece> {
        self.board[loc.index()]
    }

    // ignores bad formating of the string!
    pub fn get_piece_at(&mut self, s: &str) -> Option<Piece> {
        self.board[pos!(s).index()].take()
    }

    // ignores bad formating of the string!
    pub fn put_piece_at(&mut self, p: Piece, s: &str) {
        let pos = Pos::from_algebra(s).unwrap();
        assert!(self.piece(pos).is_none());
        self.board[pos.index()] = Some(p);
    }

    pub fn pieces(&self, f: &dyn Fn(Piece) -> bool) -> Vec<(Pos, Piece)> {
        let mut res = Vec::new();
        for ix in 0..64 {
            self.board[ix].map(|p| {
                if f(p) {
                    res.push((Pos::from_index(ix), p));
                }
            });
        }
        res
    }

    pub fn get_pieces_by_type_and_color(&self, k: PieceType, c: Color) -> Vec<Pos> {
        let q = Piece { kind: k, color: c };
        self.pieces(&|p| p == q)
            .into_iter()
            .map(|(pos, _)| pos)
            .collect()
    }

    pub fn get_pieces_by_color(&self, c: Color) -> Vec<(Pos, Piece)> {
        self.pieces(&|p| p.color == c)
    }

    pub fn occupied(&self, pos: Pos) -> bool {
        self.piece(pos).is_some()
    }

    pub fn is_en_passant_target(&self, p: Pos) -> bool {
        self.en_passant_target.map_or(false, |q| p == q)
    }

    pub fn castle_kingside_rights(&self, c: Color) -> bool {
        match c {
            Color::White => self.castle_rights[0],
            Color::Black => self.castle_rights[2],
        }
    }

    pub fn castle_queenside_rights(&self, c: Color) -> bool {
        match c {
            Color::White => self.castle_rights[1],
            Color::Black => self.castle_rights[3],
        }
    }
}

impl PartialEq for Board {
    fn eq(&self, other: &Board) -> bool {
        let mut eq = true;
        eq &= self.color_to_move == other.color_to_move;
        eq &= self.castle_rights == other.castle_rights;
        eq &= self.en_passant_target == other.en_passant_target;
        // // these properties dont affect what the next move could be
        eq &= self.halfmove_clock == other.halfmove_clock;
        eq &= self.move_number == other.move_number;
        for i in 0..64 {
            match (self.board[i], other.board[i]) {
                (Some(p), Some(q)) => eq &= p == q,
                (None, None) => {}
                _ => return false,
            }
        }
        eq
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..8 {
            write!(f, "{} [ ", 8 - i)?;
            for j in 0..8 {
                match self.board[i * 8 + j] {
                    Some(x) => write!(f, "{} ", x)?,
                    None => write!(f, "_ ")?,
                }
            }
            write!(f, "]\n")?;
        }
        write!(f, "    a b c d e f g h\n")?;
        write!(f, "{}.", self.move_number)?;

        match self.color_to_move {
            Color::White => write!(f, " White to move.")?,
            Color::Black => write!(f, " Black to move.")?,
        }

        if self.castle_rights.iter().any(|&x| x) {
            write!(f, " [")?;
            if self.castle_rights[0] {
                write!(f, "K")?;
            }
            if self.castle_rights[1] {
                write!(f, "Q")?;
            }
            if self.castle_rights[2] {
                write!(f, "k")?;
            }
            if self.castle_rights[3] {
                write!(f, "q")?;
            }
            write!(f, "]")?;
        }

        write!(f, " ({})", self.halfmove_clock)?;

        match self.en_passant_target {
            None => {}
            Some(pos) => write!(f, " ({})", pos)?,
        }

        write!(f, "\n")
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}
