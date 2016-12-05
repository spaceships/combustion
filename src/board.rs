use util::{from_algebra, to_algebra, ChessError};
use std::fmt;
use std::cmp::{min, max};
use rand::{self, Rng};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Color {
    White,
    Black,
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
    kind: PieceType,
    color: Color,
}

// const WHITE_KING: Piece = Piece { kind: PieceType::King, color: Color::White };
// const BLACK_KING: Piece = Piece { kind: PieceType::King, color: Color::White };

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Castle {
    Kingside,
    Queenside,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Pos(usize);

pub struct Board {
    board: [Option<Piece>; 64],
    pub color_to_move: Color,
    castle_rights: [bool; 4], // [ white K, white Q, black k, black q ]
    en_passant_target: Option<Pos>,
    halfmove_clock: usize,
    move_number: usize,
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
pub struct Move {
    kind: PieceType,
    from: Pos,
    to: Pos,
    takes: bool,
    en_passant: bool,
    promotion: Option<PieceType>,
    castle: Option<Castle>,
}

impl Color {
    pub fn other(&self) -> Color {
        match *self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

impl Move {
    #[allow(dead_code)]
    fn from_algebra(s: &str) -> Result<Move, ChessError> {
        if s == "O-O" {
            Ok( Move {
                kind: PieceType::King,
                from: Pos(0), to: Pos(0), takes: false,
                en_passant: false, promotion: None,
                castle: Some(Castle::Kingside),
            })
        } else if s == "O-O-O" {
            Ok( Move {
                kind: PieceType::King,
                from: Pos(0), to: Pos(0), takes: false,
                en_passant: false, promotion: None,
                castle: Some(Castle::Queenside)
            })
        } else {
            let mut cs: Vec<char> = s.chars().collect();
            let kind = match cs[0] {
                'B'|'N'|'R'|'Q'|'K' => {
                    match cs.remove(0) {
                        'B' => PieceType::Bishop,
                        'N' => PieceType::Knight,
                        'R' => PieceType::Rook,
                        'Q' => PieceType::Queen,
                        'K' => PieceType::King,
                        c   => parse_error!("[Move::from_algebra] expected one of {{B,N,R,Q,K}}, got: \"{}\"", c),
                    }
                }
                _ => PieceType::Pawn,
            };
            let from: String = cs[0..2].iter().cloned().collect();
            let to:   String = cs[3..5].iter().cloned().collect();
            let mut ep = false;
            let mut promotion = None;
            if cs.len() > 5 {
                let extras: String = cs[5..].iter().cloned().collect();
                if extras == "e.p." {
                    ep = true;
                } else if extras == "=Q" {
                    promotion = Some(PieceType::Queen);
                } else if extras == "=N" {
                    promotion = Some(PieceType::Knight);
                } else if extras == "=R" {
                    promotion = Some(PieceType::Rook);
                } else if extras == "=B" {
                    promotion = Some(PieceType::Bishop);
                } else {
                    parse_error!("[Move::from_algebra] unknown suffix: \"{}\"", extras);
                }
            }
            Ok(Move {
                kind: kind,
                from: Pos(from_algebra(&from)?),
                to:   Pos(from_algebra(&to)?),
                takes: cs[2] == 'x',
                en_passant: ep,
                promotion: promotion,
                castle: None,
            })
        }
    }

    pub fn to_xboard_format(&self, c: Color) -> String {
        match self.castle {
            Some(Castle::Kingside)  =>
                match c {
                    Color::White => return "e1g1".to_string(),
                    Color::Black => return "e8g8".to_string(),
                },
            Some(Castle::Queenside) =>
                match c {
                    Color::White => return "e1c1".to_string(),
                    Color::Black => return "e8c8".to_string(),
                },
            None => {}
        }
        format!("{}{}{}{}", self.from, self.to,
            if self.en_passant { "e.p." } else { "" },
            match self.promotion {
                Some(PieceType::Bishop) => "b",
                Some(PieceType::Knight) => "n",
                Some(PieceType::Rook)   => "r",
                Some(PieceType::Queen)  => "q",
                _ => ""
            }
        )
    }

    pub fn from_xboard_format(s: &str, b: &Board) -> Result<Move, ChessError> {
        let from = Pos::from_algebra(&s[0..2])?;
        let to   = Pos::from_algebra(&s[2..4])?;
        let p = match b.piece(from) {
            Some(p) => p,
            None => illegal_move_error!("[from_xboard_format] {}: no piece at {}!", s, from),
        };
        let q = b.piece(to);
        let mut prom = None;
        let mut ep = false;
        if s.len() > 4 {
            let extras = s[4..].to_string();
            if extras == "e.p." {
                ep = true;
            } else if extras == "q" {
                prom = Some(PieceType::Queen);
            } else if extras == "r" {
                prom = Some(PieceType::Rook);
            } else if extras == "n" {
                prom = Some(PieceType::Knight);
            } else if extras == "b" {
                prom = Some(PieceType::Bishop);
            } else {
                parse_error!("[Move::from_xboard_format] unknown suffix: \"{}\"", extras);
            }
        }
        let castle = if p.kind == PieceType::King {
            if from == Pos::from_algebra("e1")? && to == Pos::from_algebra("g1")? ||
               from == Pos::from_algebra("e8")? && to == Pos::from_algebra("g8")?
            {
                Some(Castle::Kingside)
            }

            else if from == Pos::from_algebra("e1")? && to == Pos::from_algebra("c1")? ||
                    from == Pos::from_algebra("e8")? && to == Pos::from_algebra("c8")?
            {
                Some(Castle::Queenside)
            }

            else
            {
                None
            }
        } else { None };

        let m = Move {
            kind: p.kind,
            from: from,
            to: to,
            takes: q.is_some(),
            en_passant: ep,
            promotion: prom,
            castle: castle,
        };
        Ok(m)
    }
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

    pub fn from_fen(fen: &str) -> Result<Board, ChessError> {//{{{
        let mut b = Board::new();
        let mut i = 0;
        let mut j = 0;
        let tokens: Vec<&str> = fen.split(" ").collect();

        let check = |i,j| {
            if i*8+j >= 64 {
                Err(ChessError::ParseError(format!("[from_fen] index out of bounds i={} j={}!", i, j)))
            } else {
                Ok(())
            }
        };

        // parse board
        for c in tokens[0].chars() {
            match c {
                ' ' => break,
                '/' => { i += 1; j = 0; }

                n @ '1' ... '8' => {
                    j += n.to_string().parse().expect("couldn't read number!");
                }

                'P' => { check(i,j)?; b.board[i*8+j] = Some(Piece { kind: PieceType::Pawn,   color : Color::White }); j += 1; }
                'p' => { check(i,j)?; b.board[i*8+j] = Some(Piece { kind: PieceType::Pawn,   color : Color::Black }); j += 1; }
                'B' => { check(i,j)?; b.board[i*8+j] = Some(Piece { kind: PieceType::Bishop, color : Color::White }); j += 1; }
                'b' => { check(i,j)?; b.board[i*8+j] = Some(Piece { kind: PieceType::Bishop, color : Color::Black }); j += 1; }
                'N' => { check(i,j)?; b.board[i*8+j] = Some(Piece { kind: PieceType::Knight, color : Color::White }); j += 1; }
                'n' => { check(i,j)?; b.board[i*8+j] = Some(Piece { kind: PieceType::Knight, color : Color::Black }); j += 1; }
                'R' => { check(i,j)?; b.board[i*8+j] = Some(Piece { kind: PieceType::Rook,   color : Color::White }); j += 1; }
                'r' => { check(i,j)?; b.board[i*8+j] = Some(Piece { kind: PieceType::Rook,   color : Color::Black }); j += 1; }
                'Q' => { check(i,j)?; b.board[i*8+j] = Some(Piece { kind: PieceType::Queen,  color : Color::White }); j += 1; }
                'q' => { check(i,j)?; b.board[i*8+j] = Some(Piece { kind: PieceType::Queen,  color : Color::Black }); j += 1; }
                'K' => { check(i,j)?; b.board[i*8+j] = Some(Piece { kind: PieceType::King,   color : Color::White }); j += 1; }
                'k' => { check(i,j)?; b.board[i*8+j] = Some(Piece { kind: PieceType::King,   color : Color::Black }); j += 1; }

                c => parse_error!("[from_fen] unexpected \"{}\"", c),
            }
        }

        // parse turn
        match tokens[1] {
            "w"|"W" => b.color_to_move = Color::White,
            "b"|"B" => b.color_to_move = Color::Black,
            c => parse_error!("[from_fen] unexpected \"{}\"", c),
        }

        // parse castling rights
        for c in tokens[2].chars() {
            match c {
                'K' => b.castle_rights[0] = true,
                'Q' => b.castle_rights[1] = true,
                'k' => b.castle_rights[2] = true,
                'q' => b.castle_rights[3] = true,
                '-' => {}
                c => parse_error!("[from_fen] unexpected \"{}\"", c),
            }
        }

        // parse en-passant string
        match tokens[3] {
            "-" => {}
            s   => b.en_passant_target = Some(Pos::from_algebra(s)?),
        }

        b.halfmove_clock = match tokens[4].parse() {
            Ok(c) => c,
            Err(_) => parse_error!("[from_fen] couldn't decode half move clock!"),
        };

        b.move_number = match tokens[5].parse() {
            Ok(c) => c,
            Err(_) => parse_error!("[from_fen] couldn't decode move number!"),
        };

        Ok(b)
    }
    //}}}

    pub fn initial() -> Self {
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }

    fn piece(&self, loc: Pos) -> Option<Piece> {
        self.board[loc.0]
    }

    // ignores bad formating of the string!
    fn get_piece_at(&mut self, s: &str) -> Option<Piece> {
        self.board[Pos::from_algebra(s).unwrap().0].take()
    }

    // ignores bad formating of the string!
    fn put_piece_at(&mut self, p: Piece, s: &str) {
        let pos = Pos::from_algebra(s).unwrap();
        assert!(self.piece(pos).is_none());
        self.board[pos.0] = Some(p);
    }

    fn pieces(&self, f: &Fn(Piece) -> bool) -> Vec<(Pos, Piece)> {
        let mut res = Vec::new();
        for ix in 0..64 {
            self.board[ix].map(|p| {
                if f(p) {
                    res.push((Pos(ix),p));
                }
            });
        }
        res
    }

    fn get_pieces_by_type_and_color(&self, k: PieceType, c: Color) -> Vec<Pos> {
        let q = Piece { kind: k, color: c };
        self.pieces(&|p| p == q).into_iter().map(|(pos,_)| pos).collect()
    }

    fn get_pieces_by_color(&self, c: Color) -> Vec<(Pos, Piece)> {
        self.pieces(&|p| p.color == c)
    }

    fn occupied(&self, pos: Pos) -> bool {
        self.piece(pos).is_some()
    }

    fn is_en_passant_target(&self, p: Pos) -> bool {
        self.en_passant_target.map_or(false, |q| p == q)
    }

    fn castle_kingside_rights(&self, c: Color) -> bool {
        match c {
            Color::White => self.castle_rights[0],
            Color::Black => self.castle_rights[2],
        }
    }

    fn castle_queenside_rights(&self, c: Color) -> bool {
        match c {
            Color::White => self.castle_rights[1],
            Color::Black => self.castle_rights[3],
        }
    }

    fn threatens(&self, c: Color, old: Pos) -> bool {//{{{
        for (new, piece) in self.get_pieces_by_color(c) {
            match piece.kind {
                PieceType::Pawn => {
                    match c {
                        Color::White => {
                            // omg en passant is complicated
                            if new.northeast(1).map_or(false, |ray|
                                ray == old || self.is_en_passant_target(ray) && ray.south(1).unwrap() == old) ||
                               new.northwest(1).map_or(false, |ray|
                                ray == old || self.is_en_passant_target(ray) && ray.south(1).unwrap() == old)
                            {
                                return true;
                            }
                        }
                        Color::Black => {
                            if new.southeast(1).map_or(false, |ray|
                                ray == old || self.is_en_passant_target(ray) && ray.north(1).unwrap() == old) ||
                               new.southwest(1).map_or(false, |ray|
                                ray == old || self.is_en_passant_target(ray) && ray.north(1).unwrap() == old)
                            {
                                return true;
                            }
                        }
                    }
                }

                PieceType::King => {
                    if new.mv( 1, 1).map_or(false, |ray| ray == old) ||
                       new.mv( 1, 0).map_or(false, |ray| ray == old) ||
                       new.mv( 1,-1).map_or(false, |ray| ray == old) ||
                       new.mv( 0, 1).map_or(false, |ray| ray == old) ||
                       new.mv( 0,-1).map_or(false, |ray| ray == old) ||
                       new.mv(-1, 1).map_or(false, |ray| ray == old) ||
                       new.mv(-1, 0).map_or(false, |ray| ray == old) ||
                       new.mv(-1,-1).map_or(false, |ray| ray == old)
                    {
                        return true;
                    }
                }

                PieceType::Knight => {
                    if new.mv( 1, 2).map_or(false, |ray| ray == old) ||
                       new.mv( 1,-2).map_or(false, |ray| ray == old) ||
                       new.mv(-1,-2).map_or(false, |ray| ray == old) ||
                       new.mv(-1, 2).map_or(false, |ray| ray == old) ||
                       new.mv( 2, 1).map_or(false, |ray| ray == old) ||
                       new.mv( 2,-1).map_or(false, |ray| ray == old) ||
                       new.mv(-2,-1).map_or(false, |ray| ray == old) ||
                       new.mv(-2, 1).map_or(false, |ray| ray == old)
                    {
                        return true;
                    }
                }

                PieceType::Rook => {
                    let mut ray = new;
                    while ray.north(1).is_some() {
                        ray = ray.north(1).unwrap();
                        if ray == old { return true }
                        if self.piece(ray).is_some() { break }
                    }

                    ray = new;
                    while ray.south(1).is_some() {
                        ray = ray.south(1).unwrap();
                        if ray == old { return true }
                        if self.piece(ray).is_some() { break }
                    }

                    ray = new;
                    while ray.east(1).is_some() {
                        ray = ray.east(1).unwrap();
                        if ray == old { return true }
                        if self.piece(ray).is_some() { break }
                    }

                    ray = new;
                    while ray.west(1).is_some() {
                        ray = ray.west(1).unwrap();
                        if ray == old { return true }
                        if self.piece(ray).is_some() { break }
                    }
                }

                PieceType::Bishop => {
                    let mut ray = new;
                    while ray.northeast(1).is_some() {
                        ray = ray.northeast(1).unwrap();
                        if ray == old { return true }
                        if self.piece(ray).is_some() { break }
                    }

                    ray = new;
                    while ray.northwest(1).is_some() {
                        ray = ray.northwest(1).unwrap();
                        if ray == old { return true }
                        if self.piece(ray).is_some() { break }
                    }

                    ray = new;
                    while ray.southeast(1).is_some() {
                        ray = ray.southeast(1).unwrap();
                        if ray == old { return true }
                        if self.piece(ray).is_some() { break }
                    }

                    ray = new;
                    while ray.southwest(1).is_some() {
                        ray = ray.southwest(1).unwrap();
                        if ray == old { return true }
                        if self.piece(ray).is_some() { break }
                    }
                }

                PieceType::Queen => {
                    let mut ray = new;
                    while ray.north(1).is_some() {
                        ray = ray.north(1).unwrap();
                        if ray == old { return true }
                        if self.piece(ray).is_some() { break }
                    }

                    ray = new;
                    while ray.south(1).is_some() {
                        ray = ray.south(1).unwrap();
                        if ray == old { return true }
                        if self.piece(ray).is_some() { break }
                    }

                    ray = new;
                    while ray.east(1).is_some() {
                        ray = ray.east(1).unwrap();
                        if ray == old { return true }
                        if self.piece(ray).is_some() { break }
                    }

                    ray = new;
                    while ray.west(1).is_some() {
                        ray = ray.west(1).unwrap();
                        if ray == old { return true }
                        if self.piece(ray).is_some() { break }
                    }

                    ray = new;
                    while ray.northeast(1).is_some() {
                        ray = ray.northeast(1).unwrap();
                        if ray == old { return true }
                        if self.piece(ray).is_some() { break }
                    }

                    ray = new;
                    while ray.northwest(1).is_some() {
                        ray = ray.northwest(1).unwrap();
                        if ray == old { return true }
                        if self.piece(ray).is_some() { break }
                    }

                    ray = new;
                    while ray.southeast(1).is_some() {
                        ray = ray.southeast(1).unwrap();
                        if ray == old { return true }
                        if self.piece(ray).is_some() { break }
                    }

                    ray = new;
                    while ray.southwest(1).is_some() {
                        ray = ray.southwest(1).unwrap();
                        if ray == old { return true }
                        if self.piece(ray).is_some() { break }
                    }
                }
            }
        }
        false
    }
//}}}
    pub fn make_move(&self, mv: &Move) -> Result<Board, ChessError> {//{{{
        let color = self.color_to_move;
        let mut b = Board { color_to_move: color.other(), .. *self };
        if color == Color::Black {
            b.move_number += 1;
        }
        if mv.takes || mv.kind == PieceType::Pawn {
            b.halfmove_clock = 0;
        } else {
            b.halfmove_clock += 1;
        }
        if let Some(c) = mv.castle {
            match c {
                Castle::Kingside => {
                    if !self.castle_kingside_rights(color) {
                        illegal_move_error!("[make_move] {}: kingside castle without rights!", mv);
                    }
                    match color {
                        Color::White => {
                            if self.threatens(color.other(), Pos::from_algebra("e1")?) ||
                               self.threatens(color.other(), Pos::from_algebra("f1")?) ||
                               self.threatens(color.other(), Pos::from_algebra("g1")?) {
                                illegal_move_error!("[make_move] {}: white cannot castle kingside through check!", mv);
                            }
                            if self.occupied(Pos::from_algebra("f1")?) || self.occupied(Pos::from_algebra("g1")?) {
                                illegal_move_error!("[make_move] {}: white cannot castle kingside: spaces occupied!", mv);
                            }
                            let k = match b.get_piece_at("e1") {
                                Some(p@Piece{kind: PieceType::King, color: Color::White}) => p,
                                _ => illegal_move_error!("[make_move] {}: no white king at e1!", mv),
                            };
                            let r = match b.get_piece_at("h1") {
                                Some(p@Piece{kind: PieceType::Rook, color: Color::White}) => p,
                                _ => illegal_move_error!("[make_move] {}: no white rook at h1!", mv),
                            };
                            b.put_piece_at(k, "g1");
                            b.put_piece_at(r, "f1");
                            b.castle_rights[0] = false;
                            b.castle_rights[1] = false;
                        }
                        Color::Black => {
                            if self.threatens(color.other(), Pos::from_algebra("e8")?) ||
                               self.threatens(color.other(), Pos::from_algebra("f8")?) ||
                               self.threatens(color.other(), Pos::from_algebra("g8")?) {
                                illegal_move_error!("[make_move] {} black cannot castle kingside through check!", mv);
                            }
                            if self.occupied(Pos::from_algebra("f8")?) || self.occupied(Pos::from_algebra("g8")?) {
                                illegal_move_error!("[make_move] {}: black cannot castle kingside: spaces occupied!", mv);
                            }
                            let k = match b.get_piece_at("e8") {
                                Some(p@Piece{kind: PieceType::King, color: Color::Black}) => p,
                                _ => illegal_move_error!("[make_move] {}: no black king at e8!", mv),
                            };
                            let r = match b.get_piece_at("h8") {
                                Some(p@Piece{kind: PieceType::Rook, color: Color::Black}) => p,
                                _ => illegal_move_error!("[make_move] {}: no black rook at h8!", mv),
                            };
                            b.put_piece_at(k, "g8");
                            b.put_piece_at(r, "f8");
                            b.castle_rights[2] = false;
                            b.castle_rights[3] = false;
                        }
                    }
                }
                Castle::Queenside => {
                    if !self.castle_queenside_rights(color) {
                        illegal_move_error!("[make_move] {}: queenside castle without rights!", mv);
                    }
                    match color {
                        Color::White => {
                            if self.threatens(color.other(), Pos::from_algebra("e1")?) ||
                               self.threatens(color.other(), Pos::from_algebra("d1")?) ||
                               self.threatens(color.other(), Pos::from_algebra("c1")?) {
                                illegal_move_error!("[make_move] {}: white cannot castle queenside through check!", mv);
                            }
                            if self.occupied(Pos::from_algebra("b1")?) ||
                                self.occupied(Pos::from_algebra("c1")?) ||
                                self.occupied(Pos::from_algebra("d1")?) {
                                illegal_move_error!("[make_move] {}: white cannot castle queenside: spaces occupied!", mv);
                            }
                            let k = match b.get_piece_at("e1") {
                                Some(p@Piece{kind: PieceType::King, color: Color::White}) => p,
                                _ => illegal_move_error!("[make_move] {}: no white king at e1!", mv),
                            };
                            let r = match b.get_piece_at("a1") {
                                Some(p@Piece{kind: PieceType::Rook, color: Color::White}) => p,
                                _ => illegal_move_error!("[make_move] {}: no white rook at a1!", mv),
                            };
                            b.put_piece_at(k, "c1");
                            b.put_piece_at(r, "d1");
                            b.castle_rights[0] = false;
                            b.castle_rights[1] = false;
                        }
                        Color::Black => {
                            if self.threatens(color.other(), Pos::from_algebra("e8")?) ||
                               self.threatens(color.other(), Pos::from_algebra("d8")?) ||
                               self.threatens(color.other(), Pos::from_algebra("c8")?) {
                                illegal_move_error!("[make_move] {}: black cannot castle queenside through check!", mv);
                            }
                            if self.occupied(Pos::from_algebra("b8")?) ||
                               self.occupied(Pos::from_algebra("c8")?) ||
                               self.occupied(Pos::from_algebra("d8")?) {
                                illegal_move_error!("[make_move] {}: black cannot castle queenside: spaces occupied!", mv);
                            }
                            let k = match b.get_piece_at("e8") {
                                Some(p@Piece{kind: PieceType::King, color: Color::Black}) => p,
                                _ => illegal_move_error!("[make_move] {}: no black king at e8!", mv),
                            };
                            let r = match b.get_piece_at("a8") {
                                Some(p@Piece{kind: PieceType::Rook, color: Color::Black}) => p,
                                _ => illegal_move_error!("[make_move] {}: no black rook at a8!", mv),
                            };
                            b.put_piece_at(k, "c8");
                            b.put_piece_at(r, "d8");
                            b.castle_rights[2] = false;
                            b.castle_rights[3] = false;
                        }
                    }
                }
            }
        } else if mv.en_passant {
            // check that en_passant is valid
            let ep = match b.en_passant_target.take() {
                Some(ep) => ep,
                None => illegal_move_error!("[make_move] {}: enpassant not allowed!", mv),
            };
            if ep != mv.to {
                illegal_move_error!("[make_move] {}: illegal en passant!", mv);
            }
            // check that some piece exists at mv.from
            let p = match b.board[mv.from.0].take() {
                Some(p) => p,
                None => illegal_move_error!("[make_move] {}: no piece at {}", mv, mv.from),
            };
            // check that we are moving a white piece if it is white's turn
            if p.color != color {
                illegal_move_error!("[make_move] {}: tried to move {}'s piece but it is {}'s turn!", mv, p.color, color);
            }
            // find the position of the piece we are capturing
            let target_piece_at = match color {
                Color::White => mv.to.south(1).unwrap(),
                Color::Black => mv.to.north(1).unwrap(),
            };
            // there should be no peice at the en passant target
            match b.board[mv.to.0] {
                Some(p) => board_state_error!("[make_move] {}: there should be no piece at {} but I found {}", mv, mv.to, p),
                None => {}
            }
            // remove the target piece from the board
            let q = b.board[target_piece_at.0].take();
            // check that there was actually something there
            if q.is_none() {
                illegal_move_error!("[make_move] {}: there was no piece at {}!", mv, target_piece_at);
            }
            // check that we're taking a piece of the opposite color!
            if q.map_or(false, |q| q.color == color) {
                illegal_move_error!("[make_move] {}: taking a piece of the same color!", mv);
            }
            // place the capturing piece
            b.board[mv.to.0] = Some(p);
            // reset en passant target
            b.en_passant_target = None;
        } else {
            // grab the moving/capturing piece
            let p = b.board[mv.from.0].take().expect(&format!("[make_move] {}: no piece at {}", mv, mv.from));
            // check that it is the right color
            if p.color != color {
                illegal_move_error!("[make_move] {}: tried to move {}'s piece but it is {}'s turn!", mv, p.color, color);
            }
            // set the castling rights for kings and rooks
            if p.kind == PieceType::King {
                match color {
                    Color::White => {
                        b.castle_rights[0] = false;
                        b.castle_rights[1] = false;
                    }
                    Color::Black => {
                        b.castle_rights[2] = false;
                        b.castle_rights[3] = false;
                    }
                }
            }
            if p.kind == PieceType::Rook {
                match color {
                    Color::White => {
                        if mv.from == Pos::from_algebra("h1").unwrap() { b.castle_rights[0] = false }
                        if mv.from == Pos::from_algebra("a1").unwrap() { b.castle_rights[1] = false }
                    }
                    Color::Black => {
                        if mv.from == Pos::from_algebra("h8").unwrap() { b.castle_rights[2] = false }
                        if mv.from == Pos::from_algebra("a8").unwrap() { b.castle_rights[3] = false }
                    }
                }
            }
            // grab the potentially nonexistant target piece
            let q = b.board[mv.to.0].take();
            // if the move is a capture, check that there actually was a piece there
            if mv.takes && q.is_none() {
                illegal_move_error!("[make_move] {}: taking a nonexistent piece!", mv);
            }
            // check that we're taking a piece of the opposite color
            if q.map_or(false, |q| q.color == color) {
                illegal_move_error!("[make_move] {}: {} cannot take its own pieces!", mv, color);
            }
            // possibly promote, but only for pawns
            if let Some(prom) = mv.promotion {
                if p.kind != PieceType::Pawn {
                    illegal_move_error!("[make_move] {}: only pawns can promote!", mv);
                }
                b.board[mv.to.0] = Some(Piece{ kind: prom, .. p});
            } else {
                // place moving/capturing piece
                b.board[mv.to.0] = Some(p);
            }
        }
        let kings = b.get_pieces_by_type_and_color(PieceType::King, color);
        if kings.len() == 1 && b.threatens(color.other(), kings[0]) {
            illegal_move_error!("moving into check");
        }
        Ok(b)
    }
//}}}

    pub fn legal_moves(&self) -> Result<Vec<Move>, ChessError> {//{{{
        let c = self.color_to_move;
        let mut moves = Vec::new();
        for (loc, p) in self.get_pieces_by_color(c) {
            match p.kind {
                PieceType::Pawn   => moves.extend(self.pawn_moves(loc, c)),
                PieceType::Queen  => moves.extend(self.queen_moves(loc, c)),
                PieceType::Rook   => moves.extend(self.rook_moves(loc, c)),
                PieceType::Bishop => moves.extend(self.bishop_moves(loc, c)),
                PieceType::Knight => moves.extend(self.knight_moves(loc, c)),
                PieceType::King   => moves.extend(self.king_moves(loc, c)),
            }
        }
        // filter moves where the king is in check
        let moves: Vec<Move> = moves.into_iter().filter(|m| self.make_move(m).is_ok()).collect();
        // check for checkmate, stalemate, no moves (when there are no kings, haha)
        let kings = self.get_pieces_by_type_and_color(PieceType::King, c);
        if moves.len() == 0 {
            if kings.len() == 1 && self.threatens(c.other(), kings[0]) {
                Err(ChessError::Checkmate)
            } else {
                Err(ChessError::Stalemate)
            }
        } else {
            Ok(moves)
        }
    }
//}}}
    fn pawn_moves(&self, loc: Pos, c: Color) -> Vec<Move> {//{{{
        match c {
            Color::White => self.white_pawn_moves(loc),
            Color::Black => self.black_pawn_moves(loc),
        }
    }
//}}}
    fn white_pawn_moves(&self, old: Pos) -> Vec<Move> {//{{{
        let mut moves = Vec::new();
        let m = Move {
            kind: PieceType::Pawn,
            from: old, to: old,
            takes: false,
            en_passant: false,
            promotion: None,
            castle: None,
        };


        // noncapturing reqular move
        old.north(1).map(|new| {
            if !self.occupied(new) {
                // promotion
                if new.rank_is(8) {
                    moves.push(Move{to: new, promotion: Some(PieceType::Queen), .. m});
                    moves.push(Move{to: new, promotion: Some(PieceType::Knight), .. m});
                    moves.push(Move{to: new, promotion: Some(PieceType::Rook), .. m});
                    moves.push(Move{to: new, promotion: Some(PieceType::Bishop), .. m});
                } else {
                    moves.push(Move{to: new, .. m});
                }
                if old.rank_is(2) {
                    new.north(1).map(|double| {
                        if !self.occupied(double) {
                            moves.push(Move{to: double, .. m});
                        }
                    });
                }
            }
        });

        // capturing regular moves
        old.northeast(1).map(|new| {
            if self.occupied(new) {
                if new.rank_is(8) {
                    moves.push(Move{to: new, takes: true, promotion: Some(PieceType::Queen), .. m});
                    moves.push(Move{to: new, takes: true, promotion: Some(PieceType::Knight), .. m});
                    moves.push(Move{to: new, takes: true, promotion: Some(PieceType::Rook), .. m});
                    moves.push(Move{to: new, takes: true, promotion: Some(PieceType::Bishop), .. m});
                } else {
                    moves.push(Move{to: new, takes: true, .. m});
                }
            }
            else if self.is_en_passant_target(new) {
                moves.push(Move{to: new, takes: true, en_passant: true, .. m});
            }
        });

        old.northwest(1).map(|new| {
            if self.occupied(new) {
                if new.rank_is(8) {
                    moves.push(Move{to: new, takes: true, promotion: Some(PieceType::Queen), .. m});
                    moves.push(Move{to: new, takes: true, promotion: Some(PieceType::Knight), .. m});
                    moves.push(Move{to: new, takes: true, promotion: Some(PieceType::Rook), .. m});
                    moves.push(Move{to: new, takes: true, promotion: Some(PieceType::Bishop), .. m});
                } else {
                    moves.push(Move{to: new, takes: true, .. m});
                }
            }
            else if self.is_en_passant_target(new) {
                moves.push(Move{to: new, takes: true, en_passant: true, .. m});
            }
        });

        moves
    }
//}}}
    fn black_pawn_moves(&self, old: Pos) -> Vec<Move> {//{{{
        let mut moves = Vec::new();
        let m = Move {
            kind: PieceType::Pawn,
            from: old, to: old,
            takes: false,
            en_passant: false,
            promotion: None,
            castle: None,
        };

        // noncapturing reqular move
        old.south(1).map(|new| {
            if !self.occupied(new) {
                // promotion
                if new.rank_is(1) {
                    moves.push(Move{to: new, promotion: Some(PieceType::Queen), .. m});
                    moves.push(Move{to: new, promotion: Some(PieceType::Knight), .. m});
                    moves.push(Move{to: new, promotion: Some(PieceType::Rook), .. m});
                    moves.push(Move{to: new, promotion: Some(PieceType::Bishop), .. m});
                } else {
                    moves.push(Move{to: new, .. m});
                }
                if old.rank_is(7) {
                    new.south(1).map(|double| {
                        if !self.occupied(double) {
                            moves.push(Move{to: double, .. m});
                        }
                    });
                }
            }
        });

        // capturing regular moves
        old.southeast(1).map(|new| {
            if self.occupied(new) {
                if new.rank_is(1) {
                    moves.push(Move{to: new, takes: true, promotion: Some(PieceType::Queen), .. m});
                    moves.push(Move{to: new, takes: true, promotion: Some(PieceType::Knight), .. m});
                    moves.push(Move{to: new, takes: true, promotion: Some(PieceType::Rook), .. m});
                    moves.push(Move{to: new, takes: true, promotion: Some(PieceType::Bishop), .. m});
                } else {
                    moves.push(Move{to: new, takes: true, .. m});
                }
            }
            else if self.is_en_passant_target(new) {
                moves.push(Move{to: new, takes: true, en_passant: true, .. m});
            }
        });

        old.southwest(1).map(|new| {
            if self.occupied(new) {
                if new.rank_is(1) {
                    moves.push(Move{to: new, takes: true, promotion: Some(PieceType::Queen), .. m});
                    moves.push(Move{to: new, takes: true, promotion: Some(PieceType::Knight), .. m});
                    moves.push(Move{to: new, takes: true, promotion: Some(PieceType::Rook), .. m});
                    moves.push(Move{to: new, takes: true, promotion: Some(PieceType::Bishop), .. m});
                } else {
                    moves.push(Move{to: new, takes: true, .. m});
                }
            }
            else if self.is_en_passant_target(new) {
                moves.push(Move{to: new, takes: true, en_passant: true, .. m});
            }
        });

        moves
    }
//}}}
    fn queen_moves(&self, old: Pos, c: Color) -> Vec<Move> {//{{{
        let mut new = old;
        let mut moves = Vec::new();
        let m = Move {
            kind: PieceType::Queen,
            from: old, to: new,
            takes: false,
            en_passant: false,
            promotion: None,
            castle: None,
        };

        {
            let mut mv = |new, takes| moves.push(Move{to: new, takes: takes, .. m});

            while new.north(1).is_some() {
                new = new.north(1).unwrap();
                match self.piece(new) {
                    None    => mv(new, false),
                    Some(p) => { if p.color != c { mv(new, true); } break },
                }
            }
            new = old;

            while new.south(1).is_some() {
                new = new.south(1).unwrap();
                match self.piece(new) {
                    None    => mv(new, false),
                    Some(p) => { if p.color != c { mv(new, true); } break },
                }
            }
            new = old;

            while new.east(1).is_some() {
                new = new.east(1).unwrap();
                match self.piece(new) {
                    None    => mv(new, false),
                    Some(p) => { if p.color != c { mv(new, true); } break },
                }
            }
            new = old;

            while new.west(1).is_some() {
                new = new.west(1).unwrap();
                match self.piece(new) {
                    None    => mv(new, false),
                    Some(p) => { if p.color != c { mv(new, true); } break },
                }
            }
            new = old;

            while new.northeast(1).is_some() {
                new = new.northeast(1).unwrap();
                match self.piece(new) {
                    None    => mv(new, false),
                    Some(p) => { if p.color != c { mv(new, true); } break },
                }
            }
            new = old;

            while new.northwest(1).is_some() {
                new = new.northwest(1).unwrap();
                match self.piece(new) {
                    None    => mv(new, false),
                    Some(p) => { if p.color != c { mv(new, true); } break },
                }
            }
            new = old;

            while new.southeast(1).is_some() {
                new = new.southeast(1).unwrap();
                match self.piece(new) {
                    None    => mv(new, false),
                    Some(p) => { if p.color != c { mv(new, true); } break },
                }
            }
            new = old;

            while new.southwest(1).is_some() {
                new = new.southwest(1).unwrap();
                match self.piece(new) {
                    None    => mv(new, false),
                    Some(p) => { if p.color != c { mv(new, true); } break },
                }
            }
        }
        moves
    }
//}}}
    fn rook_moves(&self, old: Pos, c: Color) -> Vec<Move> {//{{{
        let mut new = old;
        let mut moves = Vec::new();
        let m = Move {
            kind: PieceType::Rook,
            from: old,
            to: new,
            takes: false,
            en_passant: false,
            promotion: None,
            castle: None,
        };

        {
            let mut mv = |new, takes| moves.push(Move{to: new, takes: takes, .. m});

            while new.north(1).is_some() {
                new = new.north(1).unwrap();
                match self.piece(new) {
                    None    => mv(new, false),
                    Some(p) => { if p.color != c { mv(new, true); } break },
                }
            }
            new = old;

            while new.south(1).is_some() {
                new = new.south(1).unwrap();
                match self.piece(new) {
                    None    => mv(new, false),
                    Some(p) => { if p.color != c { mv(new, true); } break },
                }
            }
            new = old;

            while new.east(1).is_some() {
                new = new.east(1).unwrap();
                match self.piece(new) {
                    None    => mv(new, false),
                    Some(p) => { if p.color != c { mv(new, true); } break },
                }
            }
            new = old;

            while new.west(1).is_some() {
                new = new.west(1).unwrap();
                match self.piece(new) {
                    None    => mv(new, false),
                    Some(p) => { if p.color != c { mv(new, true); } break },
                }
            }
        }
        moves
    }
//}}}
    fn bishop_moves(&self, old: Pos, c: Color) -> Vec<Move> {//{{{
        let mut new = old;
        let mut moves = Vec::new();
        let m = Move {
            kind: PieceType::Bishop,
            from: old,
            to: new,
            takes: false,
            en_passant: false,
            promotion: None,
            castle: None,
        };

        {
            let mut mv = |new, takes| moves.push(Move{to: new, takes: takes, .. m});

            while new.northeast(1).is_some() {
                new = new.northeast(1).unwrap();
                match self.piece(new) {
                    None    => mv(new, false),
                    Some(p) => { if p.color != c { mv(new, true); } break },
                }
            }
            new = old;

            while new.northwest(1).is_some() {
                new = new.northwest(1).unwrap();
                match self.piece(new) {
                    None    => mv(new, false),
                    Some(p) => { if p.color != c { mv(new, true); } break },
                }
            }
            new = old;

            while new.southeast(1).is_some() {
                new = new.southeast(1).unwrap();
                match self.piece(new) {
                    None    => mv(new, false),
                    Some(p) => { if p.color != c { mv(new, true); } break },
                }
            }
            new = old;

            while new.southwest(1).is_some() {
                new = new.southwest(1).unwrap();
                match self.piece(new) {
                    None    => mv(new, false),
                    Some(p) => { if p.color != c { mv(new, true); } break },
                }
            }
        }
        moves
    }
//}}}
    fn knight_moves(&self, old: Pos, c: Color) -> Vec<Move> {//{{{
        let mut moves = Vec::new();
        let m = Move {
            kind: PieceType::Knight,
            from: old,
            to: old,
            takes: false,
            en_passant: false,
            promotion: None,
            castle: None,
        };
        {
            let mut mv = |vert, horiz| {
                old.mv(vert,horiz).map(|new| {
                    match self.piece(new) {
                        None => moves.push(Move{to: new, takes: false, .. m}),
                        Some(p) => if p.color != c { moves.push(Move{to: new, takes: true, .. m}) },
                    }
                });
            };
            mv(1,2);
            mv(1,-2);
            mv(-1,-2);
            mv(-1,2);
            mv(2,1);
            mv(2,-1);
            mv(-2,-1);
            mv(-2,1);
        }
        moves
    }
//}}}
    fn king_moves(&self, old: Pos, c: Color) -> Vec<Move> {//{{{
        let mut moves = Vec::new();
        let m = Move {
            kind: PieceType::King,
            from: old,
            to: old,
            takes: false,
            en_passant: false,
            promotion: None,
            castle: None,
        };
        let king_moves = [(1,0), (1,1), (1,-1), (-1,0), (-1,1), (-1,-1), (0,1), (0,-1)];
        for &(vert, horiz) in king_moves.iter() {
            old.mv(vert,horiz).map(|new| {
                match self.piece(new) {
                    None => moves.push(Move{to: new, takes: false, .. m}),
                    Some(p) => if p.color != c { moves.push(Move{to: new, takes: true, .. m}) },
                }
            });
        }

        let castle = Move {
            kind: PieceType::King,
            from: Pos(0), to: Pos(0), takes: false,
            en_passant: false, promotion: None,
            castle: None,
        };

        if self.castle_kingside_rights(c) &&
            !self.occupied(old.east(1).expect("[Board::king_moves] confusing castling rights!")) &&
            !self.occupied(old.east(2).expect("[Board::king_moves] confusing castling rights!")) &&
            !self.threatens(c.other(), old) &&
            !self.threatens(c.other(), old.east(1).unwrap())
        {
            moves.push(Move{castle: Some(Castle::Kingside),..castle});
        }

        if self.castle_queenside_rights(c) &&
            !self.occupied(old.west(1).expect("[Board::king_moves] confusing castling rights!")) &&
            !self.occupied(old.west(2).expect("[Board::king_moves] confusing castling rights!")) &&
            !self.occupied(old.west(3).expect("[Board::king_moves] confusing castling rights!")) &&
            !self.threatens(c.other(), old) &&
            !self.threatens(c.other(), old.west(1).unwrap())
        {
            moves.push(Move{castle: Some(Castle::Queenside),..castle});
        }
        moves
    }
//}}}

    // get score of board in centipawns
    pub fn score(&self, color: Color) -> isize {
        let mut score = 0;
        // let color = self.color_to_move;
        for (_, p) in self.get_pieces_by_color(color) {
            match p.kind {
                PieceType::Pawn   => score += 100,
                PieceType::Knight => score += 300,
                PieceType::Bishop => score += 300,
                PieceType::Rook   => score += 500,
                PieceType::Queen  => score += 900,
                PieceType::King   => score += 400,
            }
        }
        for (_, p) in self.get_pieces_by_color(color.other()) {
            match p.kind {
                PieceType::Pawn   => score -= 100,
                PieceType::Knight => score -= 300,
                PieceType::Bishop => score -= 300,
                PieceType::Rook   => score -= 500,
                PieceType::Queen  => score -= 900,
                PieceType::King   => score -= 400,
            }
        }
        score
    }

    pub fn best_move(&self, my_color: Color, depth: usize) -> Result<(Move, isize), ChessError> {
        // find the move with the weakest response
        let moves = self.legal_moves()?;
        let mut rng = rand::thread_rng();
        let mut best_score = isize::min_value();
        let mut best_move = None;
        for mv in moves {
            let score;
            if depth == 0 {
                score = self.make_move(&mv)?.score(my_color);
            } else {
                score = self.make_move(&mv)?.alpha_beta(my_color, depth, isize::min_value(), isize::max_value())?;
            }
            if score > best_score || (score == best_score && rng.gen())
            {
                best_move = Some(mv);
                best_score = score;
            }
        }
        Ok((best_move.unwrap(), best_score))
    }

    fn alpha_beta(&self, my_color: Color, depth: usize, alpha_in: isize, beta_in: isize)
        -> Result<isize, ChessError>
    {
        let mut alpha = alpha_in;
        let mut beta  = beta_in;
        if depth == 0 {
            return Ok(self.score(my_color));
        }
        if self.color_to_move == my_color {
            // maximizing player
            let mut v = isize::min_value();
            for mv in self.legal_moves()? {
                let score = match self.make_move(&mv)?.alpha_beta(my_color, depth - 1, alpha, beta) {
                    Err(ChessError::Checkmate) => isize::max_value(),
                    Err(ChessError::Stalemate) => 0,
                    Ok(score) => score,
                    Err(e) => return Err(e),
                };
                v     = max(v, score);
                alpha = max(alpha, v);
                if beta <= alpha {
                    break;
                }
            }
            Ok(v)
        } else {
            let mut v = isize::max_value();
            for mv in self.legal_moves()? {
                let score = match self.make_move(&mv)?.alpha_beta(my_color, depth - 1, alpha, beta) {
                    Err(ChessError::Checkmate) => isize::min_value(),
                    Err(ChessError::Stalemate) => 0,
                    Ok(score) => score,
                    Err(e) => return Err(e),
                };
                v = min(v, score);
                beta = min(beta, v);
                if beta <= alpha {
                    break;
                }
            }
            Ok(v)
        }
    }

    pub fn random_move(&self) -> Result<(Move, isize), ChessError> {
        let mut rng = rand::thread_rng();
        let ms = self.legal_moves()?;
        let i = rng.gen::<usize>() % ms.len();
        Ok((ms[i], 0))
    }
}

impl Pos {
    fn new(rank: usize, file: usize) -> Pos {
        Pos(rank * 8 + file)
    }

    fn from_algebra(s: &str) -> Result<Self, ChessError> {
        Ok(Pos(from_algebra(s)?))
    }

    fn file(&self) -> usize {
        self.0 % 8
    }

    fn rank(&self) -> usize {
        (self.0 - self.file()) / 8
    }

    #[allow(dead_code)]
    fn file_is(&self, c: char) -> bool {
        let file = c as usize - 'a' as usize;
        self.file() == file
    }

    fn rank_is(&self, c: usize) -> bool {
        let rank = 8 - c;
        self.rank() == rank
    }

    // north is negative, west is negative
    fn mv(&self, vertical: isize, horizontal: isize) -> Option<Pos> {
        let rank = self.rank() as isize + vertical;
        let file = self.file() as isize + horizontal;
        if rank >= 0 && rank < 8 && file >= 0 && file < 8 {
            Some(Pos::new(rank as usize, file as usize))
        } else {
            None
        }
    }

    fn north(&self, d: isize) -> Option<Pos> { self.mv(-d, 0) }
    fn south(&self, d: isize) -> Option<Pos> { self.mv(d,  0) }
    fn east(&self, d: isize)  -> Option<Pos> { self.mv(0,  d) }
    fn west(&self, d: isize)  -> Option<Pos> { self.mv(0, -d) }
    fn northeast(&self, d: isize) -> Option<Pos> { self.mv(-d,  d) }
    fn northwest(&self, d: isize) -> Option<Pos> { self.mv(-d, -d) }
    fn southeast(&self, d: isize) -> Option<Pos> { self.mv(d,  d) }
    fn southwest(&self, d: isize) -> Option<Pos> { self.mv(d, -d) }
}

// board impls {{{
impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Color::White => write!(f, "white"),
            Color::Black => write!(f, "black"),
        }
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..8 {
            write!(f, "{} [ ", 8-i)?;
            for j in 0..8 {
                match self.board[i*8+j] {
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
                (None, None)       => {},
                _                  => return false,
            }
        }
        eq
    }
}
//}}}
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
impl fmt::Display for Pos {//{{{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", to_algebra(self.0).unwrap())
    }
}
//}}}
impl fmt::Display for Move {//{{{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.castle {
            Some(Castle::Kingside)  => write!(f, "0-0"),
            Some(Castle::Queenside) => write!(f, "0-0-0"),
            None => {
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
                    if self.takes { "x" } else { "" },
                    self.to,
                    if self.en_passant { "e.p." } else { "" },
                )?;
                match self.promotion {
                    Some(PieceType::Bishop) => write!(f, "=B")?,
                    Some(PieceType::Knight) => write!(f, "=N")?,
                    Some(PieceType::Rook)   => write!(f, "=R")?,
                    Some(PieceType::Queen)  => write!(f, "=Q")?,
                    _ => {}
                }
                write!(f, "")
            }
        }
    }
}

//}}}

#[cfg(test)]//{{{
mod tests {
    use board::{Board, Move, Color, Pos};
    use std::collections::HashSet;
//}}}
    #[test] // initial_moves//{{{
    fn initial_moves() {
        let b = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        println!("\n{}", b);
        assert_eq!(b.legal_moves().unwrap().len(), 20);
    }
//}}}
    #[test] // white_pawn//{{{
    fn white_pawn() {
        let b = Board::from_fen("8/8/8/8/3p4/p1p5/1P1P4/8 w - - 0 1").unwrap();
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("b2-b3").unwrap());
        should_be.insert(Move::from_algebra("b2-b4").unwrap());
        should_be.insert(Move::from_algebra("b2xa3").unwrap());
        should_be.insert(Move::from_algebra("b2xc3").unwrap());
        should_be.insert(Move::from_algebra("d2xc3").unwrap());
        should_be.insert(Move::from_algebra("d2-d3").unwrap());
        let res: HashSet<Move> = b.legal_moves().unwrap().into_iter().collect();
        assert_eq!(should_be, res);
        assert_eq!(
            b.make_move(&Move::from_algebra("b2-b3").unwrap()).unwrap(),
            Board::from_fen("8/8/8/8/3p4/pPp5/3P4/8 b - - 0 1").unwrap()
        );
    }
//}}}
    #[test] // black_pawn//{{{
    fn black_pawn() {
        let b = Board::from_fen("8/2p4p/1P1P4/7P/8/8/8/8 b - - 0 1").unwrap();
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("c7-c5").unwrap());
        should_be.insert(Move::from_algebra("c7-c6").unwrap());
        should_be.insert(Move::from_algebra("c7xb6").unwrap());
        should_be.insert(Move::from_algebra("c7xd6").unwrap());
        should_be.insert(Move::from_algebra("h7-h6").unwrap());
        let res: HashSet<Move> = b.legal_moves().unwrap().into_iter().collect();
        assert_eq!(should_be, res);
        assert_eq!(
            b.make_move(&Move::from_algebra("c7-c5").unwrap()).unwrap(),
            Board::from_fen("8/7p/1P1P4/2p4P/8/8/8/8 w - - 0 2").unwrap()
        );
    }
//}}}
    #[test] // white_en_passant//{{{
    fn white_en_passant() {
        let b = Board::from_fen("8/8/8/pP6/8/8/8/8 w - a6 0 1").unwrap();
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("b5-b6").unwrap());
        should_be.insert(Move::from_algebra("b5xa6e.p.").unwrap());
        let res: HashSet<Move> = b.legal_moves().unwrap().into_iter().collect();
        assert_eq!(should_be, res);
        assert!(b.threatens(Color::White, Pos::from_algebra("a6").unwrap()));
        assert_eq!(
            b.make_move(&Move::from_algebra("b5xa6e.p.").unwrap()).unwrap(),
            Board::from_fen("8/8/P7/8/8/8/8/8 b - - 0 1").unwrap()
        );
    }
//}}}
    #[test] // black_en_passant //{{{
    fn black_en_passant() {
        let b = Board::from_fen("8/8/8/8/pP6/8/8/8 b - b3 0 1").unwrap();
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("a4-a3").unwrap());
        should_be.insert(Move::from_algebra("a4xb3e.p.").unwrap());
        let res: HashSet<Move> = b.legal_moves().unwrap().into_iter().collect();
        assert_eq!(should_be, res);
        assert!(b.threatens(Color::Black, Pos::from_algebra("b3").unwrap()));
        assert_eq!(
            b.make_move(&Move::from_algebra("a4xb3e.p.").unwrap()).unwrap(),
            Board::from_fen("8/8/8/8/8/1p6/8/8 w - - 0 2").unwrap()
        );
    }
//}}}
    #[test] // white_promotion//{{{
    fn white_promotion() {
        let b = Board::from_fen("3n4/4P3/8/8/8/8/8/8 w - - 0 1").unwrap();
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("e7-e8=Q").unwrap());
        should_be.insert(Move::from_algebra("e7-e8=N").unwrap());
        should_be.insert(Move::from_algebra("e7-e8=R").unwrap());
        should_be.insert(Move::from_algebra("e7-e8=B").unwrap());
        should_be.insert(Move::from_algebra("e7xd8=Q").unwrap());
        should_be.insert(Move::from_algebra("e7xd8=N").unwrap());
        should_be.insert(Move::from_algebra("e7xd8=R").unwrap());
        should_be.insert(Move::from_algebra("e7xd8=B").unwrap());
        let res: HashSet<Move> = b.legal_moves().unwrap().into_iter().collect();
        assert_eq!(should_be, res);
        assert_eq!(
            b.make_move(&Move::from_algebra("e7-e8=N").unwrap()).unwrap(),
            Board::from_fen("3nN4/8/8/8/8/8/8/8 b - - 0 1").unwrap()
        );
    }
//}}}
    #[test] // black_promotion//{{{
    fn black_promotion() {
        let b = Board::from_fen("8/8/8/8/8/8/3p4/4N3/ b - - 0 1").unwrap();
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("d2-d1=Q").unwrap());
        should_be.insert(Move::from_algebra("d2-d1=N").unwrap());
        should_be.insert(Move::from_algebra("d2-d1=R").unwrap());
        should_be.insert(Move::from_algebra("d2-d1=B").unwrap());
        should_be.insert(Move::from_algebra("d2xe1=Q").unwrap());
        should_be.insert(Move::from_algebra("d2xe1=N").unwrap());
        should_be.insert(Move::from_algebra("d2xe1=R").unwrap());
        should_be.insert(Move::from_algebra("d2xe1=B").unwrap());
        let res: HashSet<Move> = b.legal_moves().unwrap().into_iter().collect();
        assert_eq!(should_be, res);
        assert_eq!(
            b.make_move(&Move::from_algebra("d2xe1=B").unwrap()).unwrap(),
            Board::from_fen("8/8/8/8/8/8/8/4b3 w - - 0 2").unwrap()
        );
    }
//}}}
    #[test] // white_queen//{{{
    fn white_queen() {
        let b = Board::from_fen("3n1q3/4Q3/8/4p3/7P/p7/8/8 w - - 0 1").unwrap();
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("Qe7xd8").unwrap());
        should_be.insert(Move::from_algebra("Qe7xf8").unwrap());
        should_be.insert(Move::from_algebra("Qe7xe5").unwrap());
        should_be.insert(Move::from_algebra("Qe7xa3").unwrap());
        should_be.insert(Move::from_algebra("Qe7-e6").unwrap());
        should_be.insert(Move::from_algebra("Qe7-e8").unwrap());
        should_be.insert(Move::from_algebra("Qe7-a7").unwrap());
        should_be.insert(Move::from_algebra("Qe7-b7").unwrap());
        should_be.insert(Move::from_algebra("Qe7-c7").unwrap());
        should_be.insert(Move::from_algebra("Qe7-d7").unwrap());
        should_be.insert(Move::from_algebra("Qe7-f7").unwrap());
        should_be.insert(Move::from_algebra("Qe7-g7").unwrap());
        should_be.insert(Move::from_algebra("Qe7-h7").unwrap());
        should_be.insert(Move::from_algebra("Qe7-f6").unwrap());
        should_be.insert(Move::from_algebra("Qe7-g5").unwrap());
        should_be.insert(Move::from_algebra("Qe7-d6").unwrap());
        should_be.insert(Move::from_algebra("Qe7-c5").unwrap());
        should_be.insert(Move::from_algebra("Qe7-b4").unwrap());
        should_be.insert(Move::from_algebra("h4-h5").unwrap());
        println!("should_be=");
        for mov in should_be.iter() {
            println!("  {}", mov);
        }
        let res: HashSet<Move> = b.legal_moves().unwrap().into_iter().collect();
        assert_eq!(should_be, res);
        assert_eq!(
            b.make_move(&Move::from_algebra("Qe7-b4").unwrap()).unwrap(),
            Board::from_fen("3n1q3/8/8/4p3/1Q5P/p7/8/8 b - - 1 1").unwrap()
        );
    }
//}}}
    #[test] // black_queen//{{{
    fn black_queen() {
        let b = Board::from_fen("3N1Q3/4q3/8/4P3/7p/P7/8/8 b - - 0 1").unwrap();
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("Qe7xd8").unwrap());
        should_be.insert(Move::from_algebra("Qe7xf8").unwrap());
        should_be.insert(Move::from_algebra("Qe7xe5").unwrap());
        should_be.insert(Move::from_algebra("Qe7xa3").unwrap());
        should_be.insert(Move::from_algebra("Qe7-e6").unwrap());
        should_be.insert(Move::from_algebra("Qe7-e8").unwrap());
        should_be.insert(Move::from_algebra("Qe7-a7").unwrap());
        should_be.insert(Move::from_algebra("Qe7-b7").unwrap());
        should_be.insert(Move::from_algebra("Qe7-c7").unwrap());
        should_be.insert(Move::from_algebra("Qe7-d7").unwrap());
        should_be.insert(Move::from_algebra("Qe7-f7").unwrap());
        should_be.insert(Move::from_algebra("Qe7-g7").unwrap());
        should_be.insert(Move::from_algebra("Qe7-h7").unwrap());
        should_be.insert(Move::from_algebra("Qe7-f6").unwrap());
        should_be.insert(Move::from_algebra("Qe7-g5").unwrap());
        should_be.insert(Move::from_algebra("Qe7-d6").unwrap());
        should_be.insert(Move::from_algebra("Qe7-c5").unwrap());
        should_be.insert(Move::from_algebra("Qe7-b4").unwrap());
        should_be.insert(Move::from_algebra("h4-h3").unwrap());
        println!("should_be=");
        for mov in should_be.iter() {
            println!("  {}", mov);
        }
        let res: HashSet<Move> = b.legal_moves().unwrap().into_iter().collect();
        assert_eq!(should_be, res);
        assert_eq!(
            b.make_move(&Move::from_algebra("Qe7-b4").unwrap()).unwrap(),
            Board::from_fen("3N1Q3/8/8/4P3/1q5p/P7/8/8 w - - 1 2").unwrap()
        );
    }
//}}}
    #[test] // white_rook//{{{
    fn white_rook() {
        let b = Board::from_fen("3n1q3/4R3/8/4p3/7P/p7/8/8 w - - 0 1").unwrap();
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("Re7xe5").unwrap());
        should_be.insert(Move::from_algebra("Re7-e6").unwrap());
        should_be.insert(Move::from_algebra("Re7-e8").unwrap());
        should_be.insert(Move::from_algebra("Re7-a7").unwrap());
        should_be.insert(Move::from_algebra("Re7-b7").unwrap());
        should_be.insert(Move::from_algebra("Re7-c7").unwrap());
        should_be.insert(Move::from_algebra("Re7-d7").unwrap());
        should_be.insert(Move::from_algebra("Re7-f7").unwrap());
        should_be.insert(Move::from_algebra("Re7-g7").unwrap());
        should_be.insert(Move::from_algebra("Re7-h7").unwrap());
        should_be.insert(Move::from_algebra("h4-h5").unwrap());
        println!("should_be=");
        for mov in should_be.iter() {
            println!("  {}", mov);
        }
        let res: HashSet<Move> = b.legal_moves().unwrap().into_iter().collect();
        assert_eq!(should_be, res);
        assert_eq!(
            b.make_move(&Move::from_algebra("Re7-f7").unwrap()).unwrap(),
            Board::from_fen("3n1q3/5R2/8/4p3/7P/p7/8/8 b - - 1 1").unwrap()
        );
    }
//}}}
    #[test] // black_rook//{{{
    fn black_rook() {
        let b = Board::from_fen("3N1Q3/4r3/8/4P3/7p/P7/8/8 b - - 0 1").unwrap();
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("Re7xe5").unwrap());
        should_be.insert(Move::from_algebra("Re7-e6").unwrap());
        should_be.insert(Move::from_algebra("Re7-e8").unwrap());
        should_be.insert(Move::from_algebra("Re7-a7").unwrap());
        should_be.insert(Move::from_algebra("Re7-b7").unwrap());
        should_be.insert(Move::from_algebra("Re7-c7").unwrap());
        should_be.insert(Move::from_algebra("Re7-d7").unwrap());
        should_be.insert(Move::from_algebra("Re7-f7").unwrap());
        should_be.insert(Move::from_algebra("Re7-g7").unwrap());
        should_be.insert(Move::from_algebra("Re7-h7").unwrap());
        should_be.insert(Move::from_algebra("h4-h3").unwrap());
        println!("should_be=");
        for mov in should_be.iter() {
            println!("  {}", mov);
        }
        let res: HashSet<Move> = b.legal_moves().unwrap().into_iter().collect();
        assert_eq!(should_be, res);
        assert_eq!(
            b.make_move(&Move::from_algebra("Re7-f7").unwrap()).unwrap(),
            Board::from_fen("3N1Q3/5r2/8/4P3/7p/P7/8/8 w - - 1 2").unwrap()
        );
    }
//}}}
    #[test] // white_bishop//{{{
    fn white_bishop() {
        let b = Board::from_fen("3n1q3/4B3/8/4p3/7P/p7/8/8 w - - 0 1").unwrap();
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("Be7xd8").unwrap());
        should_be.insert(Move::from_algebra("Be7xf8").unwrap());
        should_be.insert(Move::from_algebra("Be7xa3").unwrap());
        should_be.insert(Move::from_algebra("Be7-f6").unwrap());
        should_be.insert(Move::from_algebra("Be7-g5").unwrap());
        should_be.insert(Move::from_algebra("Be7-d6").unwrap());
        should_be.insert(Move::from_algebra("Be7-c5").unwrap());
        should_be.insert(Move::from_algebra("Be7-b4").unwrap());
        should_be.insert(Move::from_algebra("h4-h5").unwrap());
        println!("should_be=");
        for mov in should_be.iter() {
            println!("  {}", mov);
        }
        let res: HashSet<Move> = b.legal_moves().unwrap().into_iter().collect();
        assert_eq!(should_be, res);
        assert_eq!(
            b.make_move(&Move::from_algebra("Be7xd8").unwrap()).unwrap(),
            Board::from_fen("3B1q3/8/8/4p3/7P/p7/8/8 b - - 0 1").unwrap()
        );
    }
//}}}
    #[test] // black_bishop//{{{
    fn black_bishop() {
        let b = Board::from_fen("3N1Q3/4b3/8/4P3/7p/P7/8/8 b - - 0 1").unwrap();
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("Be7xd8").unwrap());
        should_be.insert(Move::from_algebra("Be7xf8").unwrap());
        should_be.insert(Move::from_algebra("Be7xa3").unwrap());
        should_be.insert(Move::from_algebra("Be7-f6").unwrap());
        should_be.insert(Move::from_algebra("Be7-g5").unwrap());
        should_be.insert(Move::from_algebra("Be7-d6").unwrap());
        should_be.insert(Move::from_algebra("Be7-c5").unwrap());
        should_be.insert(Move::from_algebra("Be7-b4").unwrap());
        should_be.insert(Move::from_algebra("h4-h3").unwrap());
        println!("should_be=");
        for mov in should_be.iter() {
            println!("  {}", mov);
        }
        let res: HashSet<Move> = b.legal_moves().unwrap().into_iter().collect();
        assert_eq!(should_be, res);
        assert_eq!(
            b.make_move(&Move::from_algebra("Be7xd8").unwrap()).unwrap(),
            Board::from_fen("3b1Q3/8/8/4P3/7p/P7/8/8 w - - 0 2").unwrap()
        );
    }
//}}}
    #[test] // white_knight//{{{
    fn white_knight() {
        let b = Board::from_fen("N7/2p5/8/8/4N3/2p5/8/8 w - - 0 1").unwrap();
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("Na8xc7").unwrap());
        should_be.insert(Move::from_algebra("Na8-b6").unwrap());
        should_be.insert(Move::from_algebra("Ne4-c5").unwrap());
        should_be.insert(Move::from_algebra("Ne4xc3").unwrap());
        should_be.insert(Move::from_algebra("Ne4-d6").unwrap());
        should_be.insert(Move::from_algebra("Ne4-d2").unwrap());
        should_be.insert(Move::from_algebra("Ne4-f6").unwrap());
        should_be.insert(Move::from_algebra("Ne4-f2").unwrap());
        should_be.insert(Move::from_algebra("Ne4-g5").unwrap());
        should_be.insert(Move::from_algebra("Ne4-g3").unwrap());
        println!("should_be=");
        for mov in should_be.iter() {
            println!("  {}", mov);
        }
        let res: HashSet<Move> = b.legal_moves().unwrap().into_iter().collect();
        assert_eq!(should_be, res);
        assert_eq!(
            b.make_move(&Move::from_algebra("Ne4-c5").unwrap()).unwrap(),
            Board::from_fen("N7/2p5/8/2N5/8/2p5/8/8 b - - 1 1").unwrap()
        );
    }
//}}}
    #[test] // black_knight//{{{
    fn black_knight() {
        let b = Board::from_fen("n7/2P5/8/8/4n3/2P5/8/8 b - - 0 1").unwrap();
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("Na8xc7").unwrap());
        should_be.insert(Move::from_algebra("Na8-b6").unwrap());
        should_be.insert(Move::from_algebra("Ne4-c5").unwrap());
        should_be.insert(Move::from_algebra("Ne4xc3").unwrap());
        should_be.insert(Move::from_algebra("Ne4-d6").unwrap());
        should_be.insert(Move::from_algebra("Ne4-d2").unwrap());
        should_be.insert(Move::from_algebra("Ne4-f6").unwrap());
        should_be.insert(Move::from_algebra("Ne4-f2").unwrap());
        should_be.insert(Move::from_algebra("Ne4-g5").unwrap());
        should_be.insert(Move::from_algebra("Ne4-g3").unwrap());
        println!("should_be=");
        for mov in should_be.iter() {
            println!("  {}", mov);
        }
        let res: HashSet<Move> = b.legal_moves().unwrap().into_iter().collect();
        assert_eq!(should_be, res);
        assert_eq!(
            b.make_move(&Move::from_algebra("Ne4-c5").unwrap()).unwrap(),
            Board::from_fen("n7/2P5/8/2n5/8/2P5/8/8 w - - 1 2").unwrap()
        );
    }
//}}}
    #[test] // white_king//{{{
    fn white_king() {
        let b = Board::from_fen("n/1p6/2KP5/1p6/8/8/8/8 w - - 0 1").unwrap();
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("Kc6xb7").unwrap());
        should_be.insert(Move::from_algebra("Kc6-d7").unwrap());
        should_be.insert(Move::from_algebra("Kc6-d5").unwrap());
        should_be.insert(Move::from_algebra("Kc6xb5").unwrap());
        should_be.insert(Move::from_algebra("Kc6-c5").unwrap());
        println!("should_be=");
        for mov in should_be.iter() {
            println!("  {}", mov);
        }
        let res: HashSet<Move> = b.legal_moves().unwrap().into_iter().collect();
        assert_eq!(should_be, res);
        assert_eq!(
            b.make_move(&Move::from_algebra("Kc6xb7").unwrap()).unwrap(),
            Board::from_fen("n/1K6/3P5/1p6/8/8/8/8 b - - 0 1").unwrap()
        );
    }
//}}}
    #[test] // black_king//{{{
    fn black_king() {
        let b = Board::from_fen("N/1P6/2kp5/1P6/8/8/8/8 b - - 0 1").unwrap();
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("Kc6xb7").unwrap());
        should_be.insert(Move::from_algebra("Kc6-d7").unwrap());
        should_be.insert(Move::from_algebra("Kc6-d5").unwrap());
        should_be.insert(Move::from_algebra("Kc6xb5").unwrap());
        should_be.insert(Move::from_algebra("Kc6-c5").unwrap());
        println!("should_be=");
        for mov in should_be.iter() {
            println!("  {}", mov);
        }
        let res: HashSet<Move> = b.legal_moves().unwrap().into_iter().collect();
        assert_eq!(res, should_be);
        assert_eq!(
            b.make_move(&Move::from_algebra("Kc6xb7").unwrap()).unwrap(),
            Board::from_fen("N/1k6/3p5/1P6/8/8/8/8 w - - 0 2").unwrap()
        );
    }
    //}}}
    #[test] // white_castling//{{{
    fn white_castling() {
        let b = Board::from_fen("8/8/8/8/8/8/8/R3K2R w KQ - 0 1").unwrap();
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("O-O").unwrap());
        should_be.insert(Move::from_algebra("O-O-O").unwrap());
        println!("should_be=");
        for mov in should_be.iter() {
            println!("  {}", mov);
        }
        let res: HashSet<Move> = b.legal_moves().unwrap().into_iter().collect();
        assert!(should_be.is_subset(&res));
        assert_eq!(
            b.make_move(&Move::from_algebra("O-O").unwrap()).unwrap(),
            Board::from_fen("8/8/8/8/8/8/8/R31RK1 b - - 1 1").unwrap()
        );
        assert_eq!(
            b.make_move(&Move::from_algebra("O-O-O").unwrap()).unwrap(),
            Board::from_fen("8/8/8/8/8/8/8/2KR3R b - - 1 1").unwrap()
        );
    }
//}}}
    #[test] // white_castling2 {{{
    fn white_castling2() {
        let b = Board::from_fen("8/8/8/8/8/8/8/RN2K2R w KQ - 0 1").unwrap();
        println!("\n{}", b);
        let res: HashSet<Move> = b.legal_moves().unwrap().into_iter().collect();
        assert!(!res.contains(&Move::from_algebra("O-O-O").unwrap()));
    }
//}}}
    #[test] // black_castling//{{{
    fn black_castling() {
        let b = Board::from_fen("r3k2r/8/8/8/8/8/8/8 b kq - 0 1").unwrap();
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("O-O").unwrap());
        should_be.insert(Move::from_algebra("O-O-O").unwrap());
        let res: HashSet<Move> = b.legal_moves().unwrap().into_iter().collect();
        assert!(should_be.is_subset(&res));
        assert_eq!(
            b.make_move(&Move::from_algebra("O-O").unwrap()).unwrap(),
            Board::from_fen("r31rk1/8/8/8/8/8/8/8 w - - 1 2").unwrap()
        );
        assert_eq!(
            b.make_move(&Move::from_algebra("O-O-O").unwrap()).unwrap(),
            Board::from_fen("2kr3r/8/8/8/8/8/8/8 w - - 1 2").unwrap()
        );
    }
//}}}
    #[test] // white_castling_through_threat {{{
    #[should_panic]
    fn white_castling_through_threat() {
        // shouldn't be able to castle through threatened square!
        let b = Board::from_fen("8/8/8/8/8/5q3/8/4K2R w KQ - 0 1").unwrap();
        println!("\n{}",b);
        b.make_move(&Move::from_algebra("O-O").unwrap()).unwrap();
    }
//}}}
    #[test] // black_castling_through_threat {{{
    #[should_panic]
    fn black_castling_through_threat() {
        // shouldn't be able to castle through threatened square!
        let b = Board::from_fen("4k2r/8/5Q3/8/8/8/8/8 b kq - 0 1").unwrap();
        println!("\n{}",b);
        b.make_move(&Move::from_algebra("O-O").unwrap()).unwrap();
    }
//}}}

    #[test] // pawn score {{{
    fn pawn_score_black() {
        let b = Board::from_fen("p7/8/8/p7/8/8/8/8 b - - 0 1").unwrap();
        assert_eq!(b.score(Color::Black), 200);
    }
    // }}}
}
