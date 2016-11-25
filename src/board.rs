use std::fmt;

use util::{from_algebra, to_algebra};

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
            castle_rights: [false, false, false, false],
            en_passant_target: None,
            halfmove_clock: 0,
            move_number: 1,
        }
    }

    pub fn initial() -> Self {
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }

    pub fn from_fen(fen: &str) -> Self {//{{{
        let mut b = Board::new();
        let mut i = 0;
        let mut j = 0;
        let tokens: Vec<&str> = fen.split(" ").collect();

        // parse board
        for c in tokens[0].chars() {
            match c {
                ' ' => break,
                '/' => { i += 1; j = 0; }

                n @ '1' ... '8' => {
                    j += n.to_string().parse().expect("couldn't read number!");
                }

                'P' => { b.board[i*8+j] = Some(Piece::WhitePawn);   j += 1; }
                'p' => { b.board[i*8+j] = Some(Piece::BlackPawn);   j += 1; }
                'B' => { b.board[i*8+j] = Some(Piece::WhiteBishop); j += 1; }
                'b' => { b.board[i*8+j] = Some(Piece::BlackBishop); j += 1; }
                'N' => { b.board[i*8+j] = Some(Piece::WhiteKnight); j += 1; }
                'n' => { b.board[i*8+j] = Some(Piece::BlackKnight); j += 1; }
                'R' => { b.board[i*8+j] = Some(Piece::WhiteRook);   j += 1; }
                'r' => { b.board[i*8+j] = Some(Piece::BlackRook);   j += 1; }
                'Q' => { b.board[i*8+j] = Some(Piece::WhiteQueen);  j += 1; }
                'q' => { b.board[i*8+j] = Some(Piece::BlackQueen);  j += 1; }
                'K' => { b.board[i*8+j] = Some(Piece::WhiteKing);   j += 1; }
                'k' => { b.board[i*8+j] = Some(Piece::BlackKing);   j += 1; }

                c => panic!("unexpected \"{}\"", c),
            }
        }

        // parse turn
        match tokens[1] {
            "w"|"W" => b.white_turn = true,
            "b"|"B" => b.white_turn = false,
            c => panic!("unexpected \"{}\"", c),
        }

        // parse castling rights
        for c in tokens[2].chars() {
            match c {
                'K' => b.castle_rights[0] = true,
                'Q' => b.castle_rights[1] = true,
                'k' => b.castle_rights[2] = true,
                'q' => b.castle_rights[3] = true,
                c => panic!("unexpected \"{}\"", c),
            }
        }

        // parse en-passant string
        match tokens[3] {
            "-" => {}
            s   => b.en_passant_target = Some(from_algebra(s)),
        }

        b.halfmove_clock = tokens[4].parse().expect("couldn't decode half move clock!");
        b.move_number = tokens[5].parse().expect("couldn't decode move number!");

        b
    }
    //}}}
}

impl fmt::Display for Piece {//{{{
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
//}}}
impl fmt::Display for Board {//{{{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..8 {
            write!(f, "[ ")?;
            for j in 0..8 {
                match self.board[i*8+j] {
                    Some(x) => write!(f, "{} ", x)?,
                    None => write!(f, "_ ")?,
                }
            }
            write!(f, "]\n")?;
        }
        write!(f, "{}.", self.move_number)?;

        if self.white_turn {
            write!(f, " White to move.")?;
        } else {
            write!(f, " Black to move.")?;
        }

        if self.castle_rights.iter().any(|&x| x) {
            write!(f, " [")?;
            if self.castle_rights[0] { write!(f, "K")?; }
            if self.castle_rights[1] { write!(f, "Q")?; }
            if self.castle_rights[2] { write!(f, "k")?; }
            if self.castle_rights[3] { write!(f, "q")?; }
            write!(f, "]")?;
        }

        write!(f, " ({})", self.halfmove_clock)?;

        match self.en_passant_target {
            None => {}
            Some(pos) => write!(f, " ({})", to_algebra(pos))?,
        }

        write!(f, "\n")
    }
}
//}}}

#[cfg(test)]
mod tests {
    use board::Board;

    #[test]
    fn new_board() {
        let b = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        panic!("\n{}", b);
    }
}
