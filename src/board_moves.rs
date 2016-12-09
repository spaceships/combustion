use board::Board;
use moves::{Castle, Move};
use piece::{Color, PieceType};
use util::ChessError;
use position::Pos;

impl Board {
    pub fn moves(&self) -> Vec<Move> {
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
        moves
    }

    pub fn legal_moves(&self) -> Result<Vec<Move>, ChessError> {
        // filter moves where the king is in check
        let moves: Vec<Move> = self.moves().into_iter().filter(|m| self.make_move(m).is_ok()).collect();
        // check for checkmate, stalemate, no moves (when there are no kings, haha)
        let c = self.color_to_move;
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

    fn pawn_moves(&self, loc: Pos, c: Color) -> Vec<Move> {
        match c {
            Color::White => self.white_pawn_moves(loc),
            Color::Black => self.black_pawn_moves(loc),
        }
    }

    fn white_pawn_moves(&self, old: Pos) -> Vec<Move> {
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

    fn black_pawn_moves(&self, old: Pos) -> Vec<Move> {
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

    fn queen_moves(&self, old: Pos, c: Color) -> Vec<Move> {
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

    fn rook_moves(&self, old: Pos, c: Color) -> Vec<Move> {
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

    fn bishop_moves(&self, old: Pos, c: Color) -> Vec<Move> {
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

    fn knight_moves(&self, old: Pos, c: Color) -> Vec<Move> {
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

    fn king_moves(&self, old: Pos, c: Color) -> Vec<Move> {
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
            from: Pos::zero(), to: Pos::zero(), takes: false,
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

}
