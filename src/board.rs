use std::fmt;

use util::{from_algebra, to_algebra};

#[derive(PartialEq, Clone, Copy)]
enum Color {
    White,
    Black,
}

#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash)]
enum PieceType {
    Pawn,
    Bishop,
    Knight,
    Rook,
    Queen,
    King,
}

#[derive(Copy, Clone, PartialEq)]
struct Piece {
    kind: PieceType,
    color: Color,
}

pub struct Board {
    board: [Option<Piece>; 64],
    to_move: Color,
    castle_rights: [bool; 4], // [ white K, white Q, black k, black q ]
    en_passant_target: Option<Pos>,
    halfmove_clock: usize,
    move_number: usize,
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
struct Move {
    from: Pos,
    to: Pos,
    takes: bool,
    en_passant: bool,
    kind: PieceType,
}

impl Move {
    fn from_algebra(s: &str) -> Self {
        let mut cs: Vec<char> = s.chars().collect();
        let kind = match cs[0] {
            'B'|'N'|'R'|'Q'|'K' => {
                match cs.remove(0) {
                    'B' => PieceType::Bishop,
                    'N' => PieceType::Knight,
                    'R' => PieceType::Rook,
                    'Q' => PieceType::Queen,
                    'K' => PieceType::King,
                    _ => panic!("shouldn't be getting here ever..."),
                }
            }
            _ => PieceType::Pawn,
        };
        let from: String = cs[0..2].iter().cloned().collect();
        let to:   String = cs[3..5].iter().cloned().collect();
        let mut ep = false;
        if s.len() >= 8 {
            let ep_string: String = cs[5..9].iter().cloned().collect();
            ep = ep_string == "e.p.";
        }
        Move {
            from: Pos(from_algebra(&from)),
            to: Pos(from_algebra(&to)),
            takes: cs[2] == 'x',
            en_passant: ep,
            kind: kind,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
struct Pos(usize);

impl Board {
    pub fn new() -> Self {
        Board {
            board: [None; 64],
            to_move: Color::White,
            castle_rights: [false, false, false, false],
            en_passant_target: None,
            halfmove_clock: 0,
            move_number: 1,
        }
    }

    pub fn initial() -> Self {
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }

    fn piece(&self, loc: Pos) -> Option<Piece> {
        self.board[loc.0]
    }

    fn pieces(&self, f: &Fn(Piece) -> bool) -> Vec<(Pos, Piece)> {
        let mut res = Vec::new();
        for ix in 0..64 {
            match self.board[ix] {
                Some(p) => if f(p) { res.push((Pos(ix), p)) },
                None => {}
            }
        }
        res
    }

    fn legal_moves(&self) -> Vec<Move> {
        let c = self.to_move;
        let mut moves = Vec::new();
        for (loc, p) in self.pieces(&|p: Piece| p.color == c) {
            match p.kind {
                PieceType::Pawn => moves.extend(self.pawn_moves(loc, c)),
                _ => {}
            }
        }
        for m in moves.iter() {
            println!("{}", m);
        }
        moves
    }

    fn is_en_passant_target(&self, p: Pos) -> bool {
        match self.en_passant_target {
            None    => false,
            Some(q) => p == q,
        }
    }

    fn pawn_moves(&self, loc: Pos, c: Color) -> Vec<Move> {
        match c {
            Color::White => self.white_pawn_moves(loc),
            Color::Black => self.black_pawn_moves(loc),
        }
    }

    fn white_pawn_moves(&self, old: Pos) -> Vec<Move> {
        let mut moves = Vec::new();
        let mut m;
        for ix in 0 .. 64 {
            let new = Pos(ix);
            m = Move {
                from: old,
                to: new,
                takes: false,
                en_passant: false,
                kind: PieceType::Pawn,
            };

            // noncapturing pawn move
            if !self.occupied(new) &&
                (new.is_north_by(old, 1) || (old.rank() == 6 && new.is_north_by(old, 2)))
            {
                moves.push(m);
            }

            // capturing pawn move
            else if self.occupied(new) &&
                (new.is_northeast_by(old, 1) || new.is_northwest_by(old, 1))
            {
                m.takes = true;
                moves.push(m);
            }

            // en passant
            else if self.is_en_passant_target(new) &&
                (new.is_northeast_by(old, 1) || new.is_northwest_by(old, 1))
            {
                m.takes = true;
                m.en_passant = true;
                moves.push(m);
            }
            // TODO promotion
        }
        moves
    }


    fn black_pawn_moves(&self, old: Pos) -> Vec<Move> {
        let mut moves = Vec::new();
        let mut m;
        for ix in 0 .. 64 {
            let new = Pos(ix);
            m = Move {
                from: old,
                to: new,
                takes: false,
                en_passant: false,
                kind: PieceType::Pawn,
            };

            if !self.occupied(new) &&
                (new.is_south_by(old, 1) || (old.rank() == 1 && new.is_south_by(old, 2)))
            {
                moves.push(m);
            }

            else if self.occupied(new) &&
                (new.is_southeast_by(old, 1) || new.is_southwest_by(old, 1))
            {
                m.takes = true;
                moves.push(m);
            }

            else if self.is_en_passant_target(new) &&
                (new.is_southeast_by(old, 1) || new.is_southwest_by(old, 1))
            {
                m.takes = true;
                m.en_passant = true;
                moves.push(m);
            }
        }
        moves
    }

    fn occupied(&self, pos: Pos) -> bool {
        self.piece(pos).is_some()
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

                'P' => { b.board[i*8+j] = Some(Piece { kind: PieceType::Pawn,   color : Color::White }); j += 1; }
                'p' => { b.board[i*8+j] = Some(Piece { kind: PieceType::Pawn,   color : Color::Black }); j += 1; }
                'B' => { b.board[i*8+j] = Some(Piece { kind: PieceType::Bishop, color : Color::White }); j += 1; }
                'b' => { b.board[i*8+j] = Some(Piece { kind: PieceType::Bishop, color : Color::Black }); j += 1; }
                'N' => { b.board[i*8+j] = Some(Piece { kind: PieceType::Knight, color : Color::White }); j += 1; }
                'n' => { b.board[i*8+j] = Some(Piece { kind: PieceType::Knight, color : Color::Black }); j += 1; }
                'R' => { b.board[i*8+j] = Some(Piece { kind: PieceType::Rook,   color : Color::White }); j += 1; }
                'r' => { b.board[i*8+j] = Some(Piece { kind: PieceType::Rook,   color : Color::Black }); j += 1; }
                'Q' => { b.board[i*8+j] = Some(Piece { kind: PieceType::Queen,  color : Color::White }); j += 1; }
                'q' => { b.board[i*8+j] = Some(Piece { kind: PieceType::Queen,  color : Color::Black }); j += 1; }
                'K' => { b.board[i*8+j] = Some(Piece { kind: PieceType::King,   color : Color::White }); j += 1; }
                'k' => { b.board[i*8+j] = Some(Piece { kind: PieceType::King,   color : Color::Black }); j += 1; }

                c => panic!("unexpected \"{}\"", c),
            }
        }

        // parse turn
        match tokens[1] {
            "w"|"W" => b.to_move = Color::White,
            "b"|"B" => b.to_move = Color::Black,
            c => panic!("unexpected \"{}\"", c),
        }

        // parse castling rights
        for c in tokens[2].chars() {
            match c {
                'K' => b.castle_rights[0] = true,
                'Q' => b.castle_rights[1] = true,
                'k' => b.castle_rights[2] = true,
                'q' => b.castle_rights[3] = true,
                '-' => {}
                c => panic!("unexpected \"{}\"", c),
            }
        }

        // parse en-passant string
        match tokens[3] {
            "-" => {}
            s   => b.en_passant_target = Some(Pos::from_algebra(s)),
        }

        b.halfmove_clock = tokens[4].parse().expect("couldn't decode half move clock!");
        b.move_number = tokens[5].parse().expect("couldn't decode move number!");

        b
    }
    //}}}
}

impl Pos {
    fn from_algebra(s: &str) -> Self {
        Pos(from_algebra(s))
    }

    fn file(&self) -> usize {
        self.0 % 8
    }

    fn rank(&self) -> usize {
        (self.0 - self.file()) / 8
    }

    fn is_north_by(&self, other: Pos, dist: usize) -> bool {
        other.rank() >= dist
            && self.file() == other.file()
            && self.rank() == other.rank() - dist
    }

    fn is_northeast_by(&self, other: Pos, dist: usize) -> bool {
        other.rank() >= dist
            && self.file() == other.file() + dist
            && self.rank() == other.rank() - dist
    }

    fn is_northwest_by(&self, other: Pos, dist: usize) -> bool {
        return self.file() + dist == other.file()
            && self.rank() + dist == other.rank()
    }

    fn is_south_by(&self, other: Pos, dist: usize) -> bool {
        return self.file() == other.file()
            && self.rank() == other.rank() + dist
    }

    fn is_southeast_by(&self, other: Pos, dist: usize) -> bool {
        return self.file()  + dist == other.file()
            && other.rank() + dist == self.rank()
    }

    fn is_southwest_by(&self, other: Pos, dist: usize) -> bool {
        return other.file() + dist == self.file()
            && other.rank() + dist == self.rank()
    }
}

impl fmt::Display for Piece {//{{{
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

        match self.to_move {
            Color::White => write!(f, " White to move.")?,
            Color::Black => write!(f, " Black to move.")?,
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
            Some(pos) => write!(f, " ({})", pos)?,
        }

        write!(f, "\n")
    }
}
//}}}
impl fmt::Display for Pos {//{{{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", to_algebra(self.0))
    }
}
//}}}
impl fmt::Display for Move {//{{{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            PieceType::Bishop => write!(f, "B")?,
            PieceType::Knight => write!(f, "N")?,
            PieceType::Rook   => write!(f, "R")?,
            PieceType::Queen  => write!(f, "Q")?,
            PieceType::King   => write!(f, "K")?,
            _ => {}
        }
        write!(f, "{}{}{}{}",
            self.from,
            if self.takes { "x" } else { "-" },
            self.to,
            if self.en_passant { "e.p." } else { "" },
        )
    }
}

//}}}

#[cfg(test)]
mod tests {
    use board::{Board, Move};
    use std::collections::HashSet;

    #[test]
    fn moves() {
        let b = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        println!("\n{}", b);
        assert_eq!(b.legal_moves().len(), 10);
    }

    #[test]
    fn white_pawn_moves() {
        let b = Board::from_fen("8/8/8/8/8/p1p5/1P6/8 w - - 0 1");
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("b2-b3"));
        should_be.insert(Move::from_algebra("b2-b4"));
        should_be.insert(Move::from_algebra("b2xa3"));
        should_be.insert(Move::from_algebra("b2xc3"));
        let res: HashSet<Move> = b.legal_moves().into_iter().collect();
        assert_eq!(res, should_be);
    }

    #[test]
    fn black_pawn_moves() {
        let b = Board::from_fen("8/2p5/1P1P4/8/8/8/8/8 b - - 0 1");
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("c7-c5"));
        should_be.insert(Move::from_algebra("c7-c6"));
        should_be.insert(Move::from_algebra("c7xb6"));
        should_be.insert(Move::from_algebra("c7xd6"));
        let res: HashSet<Move> = b.legal_moves().into_iter().collect();
        assert_eq!(res, should_be);
    }

    #[test]
    fn white_en_passant() {
        let b = Board::from_fen("8/8/8/pP6/8/8/8/8 w - a6 0 1");
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("b5-b6"));
        should_be.insert(Move::from_algebra("b5xa6e.p."));
        let res: HashSet<Move> = b.legal_moves().into_iter().collect();
        assert_eq!(res, should_be);
    }

    #[test]
    fn black_en_passant() {
        let b = Board::from_fen("8/8/8/8/pP6/8/8/8 b - b3 0 1");
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("a4-a3"));
        should_be.insert(Move::from_algebra("a4xb3e.p."));
        let res: HashSet<Move> = b.legal_moves().into_iter().collect();
        assert_eq!(res, should_be);
    }

    // TODO: promotion
}
