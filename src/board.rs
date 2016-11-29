use std::fmt;

use util::{from_algebra, to_algebra};

#[derive(Debug, PartialEq, Clone, Copy)]
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

#[derive(Debug, Copy, Clone, PartialEq)]
struct Piece {
    kind: PieceType,
    color: Color,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum Castle {
    Kingside,
    Queenside,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
struct Pos(usize);

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
    kind: PieceType,
    from: Pos,
    to: Pos,
    takes: bool,
    en_passant: bool,
    promotion: Option<PieceType>,
    castle: Option<Castle>,
}

impl Color {
    fn other(&self) -> Color {
        match *self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

impl Move {
    fn from_algebra(s: &str) -> Self {
        if s == "0-0" {
            Move {
                kind: PieceType::King,
                from: Pos(0), to: Pos(0), takes: false,
                en_passant: false, promotion: None,
                castle: Some(Castle::Kingside),
            }
        } else if s == "0-0-0" {
            Move {
                kind: PieceType::King,
                from: Pos(0), to: Pos(0), takes: false,
                en_passant: false, promotion: None,
                castle: Some(Castle::Queenside)
            }
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
                        _ => panic!("shouldn't be getting here ever..."),
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
                } else if extras == "Q" {
                    promotion = Some(PieceType::Queen);
                } else if extras == "N" {
                    promotion = Some(PieceType::Knight);
                } else {
                    panic!("[Move::from_algebra] unkonwn suffix: \"{}\"", extras);
                }
            }
            Move {
                kind: kind,
                from: Pos(from_algebra(&from)),
                to: Pos(from_algebra(&to)),
                takes: cs[2] == 'x',
                en_passant: ep,
                promotion: promotion,
                castle: None,
            }
        }
    }
}

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
            self.board[ix].map(|p| {
                if f(p) {
                    res.push((Pos(ix),p));
                }
            });
        }
        res
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

    fn make_move(&self, mv: &Move) -> Board {
        let from_piece = self.piece(mv.from)
            .expect(&format!("[Board::make_move] no piece at {}!", mv.from));
        assert_eq!(from_piece.color, self.to_move);
        let mut b = Board { to_move: self.to_move.other(), .. *self };
        unimplemented!()
    }

    ////////////////////////////////////////////////////////////////////////////////
    // moves generation

    fn legal_moves(&self) -> Vec<Move> {
        let c = self.to_move;
        let mut moves = Vec::new();
        for (loc, p) in self.pieces(&|p: Piece| p.color == c) {
            match p.kind {
                PieceType::Pawn   => moves.extend(self.pawn_moves(loc, c)),
                PieceType::Queen  => moves.extend(self.queen_moves(loc, c)),
                PieceType::Rook   => moves.extend(self.rook_moves(loc, c)),
                PieceType::Bishop => moves.extend(self.bishop_moves(loc, c)),
                PieceType::Knight => moves.extend(self.knight_moves(loc, c)),
                PieceType::King   => moves.extend(self.king_moves(loc, c)),
            }
        }
        for m in moves.iter() {
            println!("{}", m);
        }
        moves
    }

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
            !self.occupied(old.east(2).expect("[Board::king_moves] confusing castling rights!"))
        {
            moves.push(Move{castle: Some(Castle::Kingside),..castle});
        }

        if self.castle_queenside_rights(c) &&
            !self.occupied(old.west(1).expect("[Board::king_moves] confusing castling rights!")) &&
            !self.occupied(old.west(2).expect("[Board::king_moves] confusing castling rights!")) &&
            !self.occupied(old.west(3).expect("[Board::king_moves] confusing castling rights!"))
        {
            moves.push(Move{castle: Some(Castle::Queenside),..castle});
        }
        moves
    }
//}}}

    pub fn from_fen(fen: &str) -> Self {//{{{
        let mut b = Board::new();
        let mut i = 0;
        let mut j = 0;
        let tokens: Vec<&str> = fen.split(" ").collect();

        let check = |i,j| {
            if i*8+j >= 64 {
                let s = format!("[Board::from_fen] index out of bounds i={} j={}!", i, j);
                panic!(s);
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

                'P' => { check(i,j); b.board[i*8+j] = Some(Piece { kind: PieceType::Pawn,   color : Color::White }); j += 1; }
                'p' => { check(i,j); b.board[i*8+j] = Some(Piece { kind: PieceType::Pawn,   color : Color::Black }); j += 1; }
                'B' => { check(i,j); b.board[i*8+j] = Some(Piece { kind: PieceType::Bishop, color : Color::White }); j += 1; }
                'b' => { check(i,j); b.board[i*8+j] = Some(Piece { kind: PieceType::Bishop, color : Color::Black }); j += 1; }
                'N' => { check(i,j); b.board[i*8+j] = Some(Piece { kind: PieceType::Knight, color : Color::White }); j += 1; }
                'n' => { check(i,j); b.board[i*8+j] = Some(Piece { kind: PieceType::Knight, color : Color::Black }); j += 1; }
                'R' => { check(i,j); b.board[i*8+j] = Some(Piece { kind: PieceType::Rook,   color : Color::White }); j += 1; }
                'r' => { check(i,j); b.board[i*8+j] = Some(Piece { kind: PieceType::Rook,   color : Color::Black }); j += 1; }
                'Q' => { check(i,j); b.board[i*8+j] = Some(Piece { kind: PieceType::Queen,  color : Color::White }); j += 1; }
                'q' => { check(i,j); b.board[i*8+j] = Some(Piece { kind: PieceType::Queen,  color : Color::Black }); j += 1; }
                'K' => { check(i,j); b.board[i*8+j] = Some(Piece { kind: PieceType::King,   color : Color::White }); j += 1; }
                'k' => { check(i,j); b.board[i*8+j] = Some(Piece { kind: PieceType::King,   color : Color::Black }); j += 1; }

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
    fn new(rank: usize, file: usize) -> Pos {
        Pos(rank * 8 + file)
    }

    fn from_algebra(s: &str) -> Self {
        Pos(from_algebra(s))
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
                    if self.takes { "x" } else { "-" },
                    self.to,
                    if self.en_passant { "e.p." } else { "" },
                )?;
                match self.promotion {
                    Some(PieceType::Bishop) => write!(f, "B")?,
                    Some(PieceType::Knight) => write!(f, "N")?,
                    Some(PieceType::Rook)   => write!(f, "R")?,
                    Some(PieceType::Queen)  => write!(f, "Q")?,
                    Some(PieceType::King)   => write!(f, "K")?,
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
    use board::{Board, Move};
    use std::collections::HashSet;
//}}}
    #[test] // initial_moves//{{{
    fn initial_moves() {
        let b = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        println!("\n{}", b);
        assert_eq!(b.legal_moves().len(), 20);
    }
//}}}
    #[test] // white_pawn//{{{
    fn white_pawn() {
        let b = Board::from_fen("8/8/8/8/3p4/p1p5/1P1P4/8 w - - 0 1");
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("b2-b3"));
        should_be.insert(Move::from_algebra("b2-b4"));
        should_be.insert(Move::from_algebra("b2xa3"));
        should_be.insert(Move::from_algebra("b2xc3"));
        should_be.insert(Move::from_algebra("d2xc3"));
        should_be.insert(Move::from_algebra("d2-d3"));
        let res: HashSet<Move> = b.legal_moves().into_iter().collect();
        assert_eq!(should_be, res);
    }
//}}}
    #[test] // black_pawn//{{{
    fn black_pawn() {
        let b = Board::from_fen("8/2p4p/1P1P4/7P/8/8/8/8 b - - 0 1");
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("c7-c5"));
        should_be.insert(Move::from_algebra("c7-c6"));
        should_be.insert(Move::from_algebra("c7xb6"));
        should_be.insert(Move::from_algebra("c7xd6"));
        should_be.insert(Move::from_algebra("h7-h6"));
        let res: HashSet<Move> = b.legal_moves().into_iter().collect();
        assert_eq!(should_be, res);
    }
//}}}
    #[test] // white_en_passant//{{{
    fn white_en_passant() {
        let b = Board::from_fen("8/8/8/pP6/8/8/8/8 w - a6 0 1");
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("b5-b6"));
        should_be.insert(Move::from_algebra("b5xa6e.p."));
        let res: HashSet<Move> = b.legal_moves().into_iter().collect();
        assert_eq!(should_be, res);
    }
//}}}
    #[test] // black_en_passant //{{{
    fn black_en_passant() {
        let b = Board::from_fen("8/8/8/8/pP6/8/8/8 b - b3 0 1");
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("a4-a3"));
        should_be.insert(Move::from_algebra("a4xb3e.p."));
        let res: HashSet<Move> = b.legal_moves().into_iter().collect();
        assert_eq!(should_be, res);
    }
//}}}
    #[test] // white_promotion//{{{
    fn white_promotion() {
        let b = Board::from_fen("3n4/4P3/8/8/8/8/8/8 w - - 0 1");
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("e7-e8Q"));
        should_be.insert(Move::from_algebra("e7-e8N"));
        should_be.insert(Move::from_algebra("e7xd8Q"));
        should_be.insert(Move::from_algebra("e7xd8N"));
        let res: HashSet<Move> = b.legal_moves().into_iter().collect();
        assert_eq!(should_be, res);
    }
//}}}
    #[test] // black_promotion//{{{
    fn black_promotion() {
        let b = Board::from_fen("8/8/8/8/8/8/3p4/4N3/ b - - 0 1");
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("d2-d1Q"));
        should_be.insert(Move::from_algebra("d2-d1N"));
        should_be.insert(Move::from_algebra("d2xe1Q"));
        should_be.insert(Move::from_algebra("d2xe1N"));
        let res: HashSet<Move> = b.legal_moves().into_iter().collect();
        assert_eq!(should_be, res);
    }
//}}}
    #[test] // white_queen//{{{
    fn white_queen() {
        let b = Board::from_fen("3n1q3/4Q3/8/4p3/7P/p7/8/8 w - - 0 1");
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("Qe7xd8"));
        should_be.insert(Move::from_algebra("Qe7xf8"));
        should_be.insert(Move::from_algebra("Qe7xe5"));
        should_be.insert(Move::from_algebra("Qe7xa3"));
        should_be.insert(Move::from_algebra("Qe7-e6"));
        should_be.insert(Move::from_algebra("Qe7-e8"));
        should_be.insert(Move::from_algebra("Qe7-a7"));
        should_be.insert(Move::from_algebra("Qe7-b7"));
        should_be.insert(Move::from_algebra("Qe7-c7"));
        should_be.insert(Move::from_algebra("Qe7-d7"));
        should_be.insert(Move::from_algebra("Qe7-f7"));
        should_be.insert(Move::from_algebra("Qe7-g7"));
        should_be.insert(Move::from_algebra("Qe7-h7"));
        should_be.insert(Move::from_algebra("Qe7-f6"));
        should_be.insert(Move::from_algebra("Qe7-g5"));
        should_be.insert(Move::from_algebra("Qe7-d6"));
        should_be.insert(Move::from_algebra("Qe7-c5"));
        should_be.insert(Move::from_algebra("Qe7-b4"));
        should_be.insert(Move::from_algebra("h4-h5"));
        println!("should_be=");
        for mov in should_be.iter() {
            println!("  {}", mov);
        }
        let res: HashSet<Move> = b.legal_moves().into_iter().collect();
        assert_eq!(should_be, res);
    }
//}}}
    #[test] // black_queen//{{{
    fn black_queen() {
        let b = Board::from_fen("3N1Q3/4q3/8/4P3/7p/P7/8/8 b - - 0 1");
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("Qe7xd8"));
        should_be.insert(Move::from_algebra("Qe7xf8"));
        should_be.insert(Move::from_algebra("Qe7xe5"));
        should_be.insert(Move::from_algebra("Qe7xa3"));
        should_be.insert(Move::from_algebra("Qe7-e6"));
        should_be.insert(Move::from_algebra("Qe7-e8"));
        should_be.insert(Move::from_algebra("Qe7-a7"));
        should_be.insert(Move::from_algebra("Qe7-b7"));
        should_be.insert(Move::from_algebra("Qe7-c7"));
        should_be.insert(Move::from_algebra("Qe7-d7"));
        should_be.insert(Move::from_algebra("Qe7-f7"));
        should_be.insert(Move::from_algebra("Qe7-g7"));
        should_be.insert(Move::from_algebra("Qe7-h7"));
        should_be.insert(Move::from_algebra("Qe7-f6"));
        should_be.insert(Move::from_algebra("Qe7-g5"));
        should_be.insert(Move::from_algebra("Qe7-d6"));
        should_be.insert(Move::from_algebra("Qe7-c5"));
        should_be.insert(Move::from_algebra("Qe7-b4"));
        should_be.insert(Move::from_algebra("h4-h3"));
        println!("should_be=");
        for mov in should_be.iter() {
            println!("  {}", mov);
        }
        let res: HashSet<Move> = b.legal_moves().into_iter().collect();
        assert_eq!(should_be, res);
    }
//}}}
    #[test] // white_rook//{{{
    fn white_rook() {
        let b = Board::from_fen("3n1q3/4R3/8/4p3/7P/p7/8/8 w - - 0 1");
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("Re7xe5"));
        should_be.insert(Move::from_algebra("Re7-e6"));
        should_be.insert(Move::from_algebra("Re7-e8"));
        should_be.insert(Move::from_algebra("Re7-a7"));
        should_be.insert(Move::from_algebra("Re7-b7"));
        should_be.insert(Move::from_algebra("Re7-c7"));
        should_be.insert(Move::from_algebra("Re7-d7"));
        should_be.insert(Move::from_algebra("Re7-f7"));
        should_be.insert(Move::from_algebra("Re7-g7"));
        should_be.insert(Move::from_algebra("Re7-h7"));
        should_be.insert(Move::from_algebra("h4-h5"));
        println!("should_be=");
        for mov in should_be.iter() {
            println!("  {}", mov);
        }
        let res: HashSet<Move> = b.legal_moves().into_iter().collect();
        assert_eq!(should_be, res);
    }
//}}}
    #[test] // black_rook//{{{
    fn black_rook() {
        let b = Board::from_fen("3N1Q3/4r3/8/4P3/7p/P7/8/8 b - - 0 1");
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("Re7xe5"));
        should_be.insert(Move::from_algebra("Re7-e6"));
        should_be.insert(Move::from_algebra("Re7-e8"));
        should_be.insert(Move::from_algebra("Re7-a7"));
        should_be.insert(Move::from_algebra("Re7-b7"));
        should_be.insert(Move::from_algebra("Re7-c7"));
        should_be.insert(Move::from_algebra("Re7-d7"));
        should_be.insert(Move::from_algebra("Re7-f7"));
        should_be.insert(Move::from_algebra("Re7-g7"));
        should_be.insert(Move::from_algebra("Re7-h7"));
        should_be.insert(Move::from_algebra("h4-h3"));
        println!("should_be=");
        for mov in should_be.iter() {
            println!("  {}", mov);
        }
        let res: HashSet<Move> = b.legal_moves().into_iter().collect();
        assert_eq!(should_be, res);
    }
//}}}
    #[test] // white_bishop//{{{
    fn white_bishop() {
        let b = Board::from_fen("3n1q3/4B3/8/4p3/7P/p7/8/8 w - - 0 1");
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("Be7xd8"));
        should_be.insert(Move::from_algebra("Be7xf8"));
        should_be.insert(Move::from_algebra("Be7xa3"));
        should_be.insert(Move::from_algebra("Be7-f6"));
        should_be.insert(Move::from_algebra("Be7-g5"));
        should_be.insert(Move::from_algebra("Be7-d6"));
        should_be.insert(Move::from_algebra("Be7-c5"));
        should_be.insert(Move::from_algebra("Be7-b4"));
        should_be.insert(Move::from_algebra("h4-h5"));
        println!("should_be=");
        for mov in should_be.iter() {
            println!("  {}", mov);
        }
        let res: HashSet<Move> = b.legal_moves().into_iter().collect();
        assert_eq!(should_be, res);
    }
//}}}
    #[test] // black_bishop//{{{
    fn black_bishop() {
        let b = Board::from_fen("3N1Q3/4b3/8/4P3/7p/P7/8/8 b - - 0 1");
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("Be7xd8"));
        should_be.insert(Move::from_algebra("Be7xf8"));
        should_be.insert(Move::from_algebra("Be7xa3"));
        should_be.insert(Move::from_algebra("Be7-f6"));
        should_be.insert(Move::from_algebra("Be7-g5"));
        should_be.insert(Move::from_algebra("Be7-d6"));
        should_be.insert(Move::from_algebra("Be7-c5"));
        should_be.insert(Move::from_algebra("Be7-b4"));
        should_be.insert(Move::from_algebra("h4-h3"));
        println!("should_be=");
        for mov in should_be.iter() {
            println!("  {}", mov);
        }
        let res: HashSet<Move> = b.legal_moves().into_iter().collect();
        assert_eq!(should_be, res);
    }
//}}}
    #[test] // white_knight//{{{
    fn white_knight() {
        let b = Board::from_fen("N7/2p5/8/8/4N3/2p5/8/8 w - - 0 1");
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("Na8xc7"));
        should_be.insert(Move::from_algebra("Na8-b6"));
        should_be.insert(Move::from_algebra("Ne4-c5"));
        should_be.insert(Move::from_algebra("Ne4xc3"));
        should_be.insert(Move::from_algebra("Ne4-d6"));
        should_be.insert(Move::from_algebra("Ne4-d2"));
        should_be.insert(Move::from_algebra("Ne4-f6"));
        should_be.insert(Move::from_algebra("Ne4-f2"));
        should_be.insert(Move::from_algebra("Ne4-g5"));
        should_be.insert(Move::from_algebra("Ne4-g3"));
        println!("should_be=");
        for mov in should_be.iter() {
            println!("  {}", mov);
        }
        let res: HashSet<Move> = b.legal_moves().into_iter().collect();
        assert_eq!(should_be, res);
    }
//}}}
    #[test] // black_knight//{{{
    fn black_knight() {
        let b = Board::from_fen("n7/2P5/8/8/4n3/2P5/8/8 b - - 0 1");
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("Na8xc7"));
        should_be.insert(Move::from_algebra("Na8-b6"));
        should_be.insert(Move::from_algebra("Ne4-c5"));
        should_be.insert(Move::from_algebra("Ne4xc3"));
        should_be.insert(Move::from_algebra("Ne4-d6"));
        should_be.insert(Move::from_algebra("Ne4-d2"));
        should_be.insert(Move::from_algebra("Ne4-f6"));
        should_be.insert(Move::from_algebra("Ne4-f2"));
        should_be.insert(Move::from_algebra("Ne4-g5"));
        should_be.insert(Move::from_algebra("Ne4-g3"));
        println!("should_be=");
        for mov in should_be.iter() {
            println!("  {}", mov);
        }
        let res: HashSet<Move> = b.legal_moves().into_iter().collect();
        assert_eq!(should_be, res);
    }
//}}}
    #[test] // white_king//{{{
    fn white_king() {
        let b = Board::from_fen("n/1p6/2KP5/1p6/8/8/8/8 w - - 0 1");
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("Kc6xb7"));
        should_be.insert(Move::from_algebra("Kc6-c7"));
        should_be.insert(Move::from_algebra("Kc6-d7"));
        should_be.insert(Move::from_algebra("Kc6-b6"));
        should_be.insert(Move::from_algebra("Kc6-d5"));
        should_be.insert(Move::from_algebra("Kc6xb5"));
        should_be.insert(Move::from_algebra("Kc6-c5"));
        should_be.insert(Move::from_algebra("d6-d7"));
        println!("should_be=");
        for mov in should_be.iter() {
            println!("  {}", mov);
        }
        let res: HashSet<Move> = b.legal_moves().into_iter().collect();
        assert_eq!(should_be, res);
    }
//}}}
    #[test] // black_king//{{{
    fn black_king() {
        let b = Board::from_fen("N/1P6/2kp5/1P6/8/8/8/8 b - - 0 1");
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("d6-d5"));
        should_be.insert(Move::from_algebra("Kc6xb7"));
        should_be.insert(Move::from_algebra("Kc6-c7"));
        should_be.insert(Move::from_algebra("Kc6-d7"));
        should_be.insert(Move::from_algebra("Kc6-b6"));
        should_be.insert(Move::from_algebra("Kc6-d5"));
        should_be.insert(Move::from_algebra("Kc6xb5"));
        should_be.insert(Move::from_algebra("Kc6-c5"));
        println!("should_be=");
        for mov in should_be.iter() {
            println!("  {}", mov);
        }
        let res: HashSet<Move> = b.legal_moves().into_iter().collect();
        assert_eq!(res, should_be);
    }
    //}}}
    #[test] // white_castling//{{{
    fn white_castling() {
        let b = Board::from_fen("8/8/8/8/8/8/8/R3K2R w KQ - 0 1");
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("0-0"));
        should_be.insert(Move::from_algebra("0-0-0"));
        println!("should_be=");
        for mov in should_be.iter() {
            println!("  {}", mov);
        }
        let res: HashSet<Move> = b.legal_moves().into_iter().collect();
        assert!(should_be.is_subset(&res));
    }
//}}}
    #[test] // black_castling//{{{
    fn black_castling() {
        let b = Board::from_fen("r3k2r/8/8/8/8/8/8/8 b kq - 0 1");
        println!("\n{}", b);
        let mut should_be = HashSet::new();
        should_be.insert(Move::from_algebra("0-0"));
        should_be.insert(Move::from_algebra("0-0-0"));
        println!("should_be=");
        for mov in should_be.iter() {
            println!("  {}", mov);
        }
        let res: HashSet<Move> = b.legal_moves().into_iter().collect();
        assert!(should_be.is_subset(&res));
    }
//}}}
}
