use crate::board::Board;
use crate::moves::{Castle, Move};
use crate::piece::{Color, Piece, PieceType};
use crate::position::Pos;
use crate::util::ChessError;

impl Board {
    pub fn moves(&self) -> Vec<Move> {
        let c = self.color_to_move;
        let mut moves = Vec::new();
        for (loc, p) in self.get_pieces_by_color(c) {
            match p.kind {
                PieceType::Pawn => moves.extend(self.pawn_moves(loc, c)),
                PieceType::Queen => moves.extend(self.queen_moves(loc, c)),
                PieceType::Rook => moves.extend(self.rook_moves(loc, c)),
                PieceType::Bishop => moves.extend(self.bishop_moves(loc, c)),
                PieceType::Knight => moves.extend(self.knight_moves(loc, c)),
                PieceType::King => moves.extend(self.king_moves(loc, c)),
            }
        }
        moves
    }

    pub fn legal_moves(&self) -> Result<Vec<Move>, ChessError> {
        // filter moves where the king is in check
        let mut moves: Vec<Move> = self
            .moves()
            .into_iter()
            .filter(|m| self.make_move(m).is_ok())
            .collect();
        moves.sort();
        // check for checkmate, stalemate, no moves (when there are no kings, haha)
        let c = self.color_to_move;
        let kings = self.get_pieces_by_type_and_color(PieceType::King, c);
        if moves.len() == 0 {
            if kings.len() == 1 && self.color_threatens(c.other(), kings[0]) {
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
            from: old,
            to: old,
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
                    moves.push(Move {
                        to: new,
                        promotion: Some(PieceType::Queen),
                        ..m
                    });
                    moves.push(Move {
                        to: new,
                        promotion: Some(PieceType::Knight),
                        ..m
                    });
                    moves.push(Move {
                        to: new,
                        promotion: Some(PieceType::Rook),
                        ..m
                    });
                    moves.push(Move {
                        to: new,
                        promotion: Some(PieceType::Bishop),
                        ..m
                    });
                } else {
                    moves.push(Move { to: new, ..m });
                }
                if old.rank_is(2) {
                    new.north(1).map(|double| {
                        if !self.occupied(double) {
                            moves.push(Move { to: double, ..m });
                        }
                    });
                }
            }
        });

        // capturing regular moves
        old.northeast(1).map(|new| {
            if self.occupied(new) {
                if new.rank_is(8) {
                    moves.push(Move {
                        to: new,
                        takes: true,
                        promotion: Some(PieceType::Queen),
                        ..m
                    });
                    moves.push(Move {
                        to: new,
                        takes: true,
                        promotion: Some(PieceType::Knight),
                        ..m
                    });
                    moves.push(Move {
                        to: new,
                        takes: true,
                        promotion: Some(PieceType::Rook),
                        ..m
                    });
                    moves.push(Move {
                        to: new,
                        takes: true,
                        promotion: Some(PieceType::Bishop),
                        ..m
                    });
                } else {
                    moves.push(Move {
                        to: new,
                        takes: true,
                        ..m
                    });
                }
            } else if self.is_en_passant_target(new) {
                moves.push(Move {
                    to: new,
                    takes: true,
                    en_passant: true,
                    ..m
                });
            }
        });

        old.northwest(1).map(|new| {
            if self.occupied(new) {
                if new.rank_is(8) {
                    moves.push(Move {
                        to: new,
                        takes: true,
                        promotion: Some(PieceType::Queen),
                        ..m
                    });
                    moves.push(Move {
                        to: new,
                        takes: true,
                        promotion: Some(PieceType::Knight),
                        ..m
                    });
                    moves.push(Move {
                        to: new,
                        takes: true,
                        promotion: Some(PieceType::Rook),
                        ..m
                    });
                    moves.push(Move {
                        to: new,
                        takes: true,
                        promotion: Some(PieceType::Bishop),
                        ..m
                    });
                } else {
                    moves.push(Move {
                        to: new,
                        takes: true,
                        ..m
                    });
                }
            } else if self.is_en_passant_target(new) {
                moves.push(Move {
                    to: new,
                    takes: true,
                    en_passant: true,
                    ..m
                });
            }
        });

        moves
    }

    fn black_pawn_moves(&self, old: Pos) -> Vec<Move> {
        let mut moves = Vec::new();
        let m = Move {
            kind: PieceType::Pawn,
            from: old,
            to: old,
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
                    moves.push(Move {
                        to: new,
                        promotion: Some(PieceType::Queen),
                        ..m
                    });
                    moves.push(Move {
                        to: new,
                        promotion: Some(PieceType::Knight),
                        ..m
                    });
                    moves.push(Move {
                        to: new,
                        promotion: Some(PieceType::Rook),
                        ..m
                    });
                    moves.push(Move {
                        to: new,
                        promotion: Some(PieceType::Bishop),
                        ..m
                    });
                } else {
                    moves.push(Move { to: new, ..m });
                }
                if old.rank_is(7) {
                    new.south(1).map(|double| {
                        if !self.occupied(double) {
                            moves.push(Move { to: double, ..m });
                        }
                    });
                }
            }
        });

        // capturing regular moves
        old.southeast(1).map(|new| {
            if self.occupied(new) {
                if new.rank_is(1) {
                    moves.push(Move {
                        to: new,
                        takes: true,
                        promotion: Some(PieceType::Queen),
                        ..m
                    });
                    moves.push(Move {
                        to: new,
                        takes: true,
                        promotion: Some(PieceType::Knight),
                        ..m
                    });
                    moves.push(Move {
                        to: new,
                        takes: true,
                        promotion: Some(PieceType::Rook),
                        ..m
                    });
                    moves.push(Move {
                        to: new,
                        takes: true,
                        promotion: Some(PieceType::Bishop),
                        ..m
                    });
                } else {
                    moves.push(Move {
                        to: new,
                        takes: true,
                        ..m
                    });
                }
            } else if self.is_en_passant_target(new) {
                moves.push(Move {
                    to: new,
                    takes: true,
                    en_passant: true,
                    ..m
                });
            }
        });

        old.southwest(1).map(|new| {
            if self.occupied(new) {
                if new.rank_is(1) {
                    moves.push(Move {
                        to: new,
                        takes: true,
                        promotion: Some(PieceType::Queen),
                        ..m
                    });
                    moves.push(Move {
                        to: new,
                        takes: true,
                        promotion: Some(PieceType::Knight),
                        ..m
                    });
                    moves.push(Move {
                        to: new,
                        takes: true,
                        promotion: Some(PieceType::Rook),
                        ..m
                    });
                    moves.push(Move {
                        to: new,
                        takes: true,
                        promotion: Some(PieceType::Bishop),
                        ..m
                    });
                } else {
                    moves.push(Move {
                        to: new,
                        takes: true,
                        ..m
                    });
                }
            } else if self.is_en_passant_target(new) {
                moves.push(Move {
                    to: new,
                    takes: true,
                    en_passant: true,
                    ..m
                });
            }
        });

        moves
    }

    fn queen_moves(&self, old: Pos, c: Color) -> Vec<Move> {
        let mut new = old;
        let mut moves = Vec::new();
        let m = Move {
            kind: PieceType::Queen,
            from: old,
            to: new,
            takes: false,
            en_passant: false,
            promotion: None,
            castle: None,
        };

        {
            let mut mv = |new, takes| {
                moves.push(Move {
                    to: new,
                    takes: takes,
                    ..m
                })
            };

            while new.north(1).is_some() {
                new = new.north(1).unwrap();
                match self.piece(new) {
                    None => mv(new, false),
                    Some(p) => {
                        if p.color != c {
                            mv(new, true);
                        }
                        break;
                    }
                }
            }
            new = old;

            while new.south(1).is_some() {
                new = new.south(1).unwrap();
                match self.piece(new) {
                    None => mv(new, false),
                    Some(p) => {
                        if p.color != c {
                            mv(new, true);
                        }
                        break;
                    }
                }
            }
            new = old;

            while new.east(1).is_some() {
                new = new.east(1).unwrap();
                match self.piece(new) {
                    None => mv(new, false),
                    Some(p) => {
                        if p.color != c {
                            mv(new, true);
                        }
                        break;
                    }
                }
            }
            new = old;

            while new.west(1).is_some() {
                new = new.west(1).unwrap();
                match self.piece(new) {
                    None => mv(new, false),
                    Some(p) => {
                        if p.color != c {
                            mv(new, true);
                        }
                        break;
                    }
                }
            }
            new = old;

            while new.northeast(1).is_some() {
                new = new.northeast(1).unwrap();
                match self.piece(new) {
                    None => mv(new, false),
                    Some(p) => {
                        if p.color != c {
                            mv(new, true);
                        }
                        break;
                    }
                }
            }
            new = old;

            while new.northwest(1).is_some() {
                new = new.northwest(1).unwrap();
                match self.piece(new) {
                    None => mv(new, false),
                    Some(p) => {
                        if p.color != c {
                            mv(new, true);
                        }
                        break;
                    }
                }
            }
            new = old;

            while new.southeast(1).is_some() {
                new = new.southeast(1).unwrap();
                match self.piece(new) {
                    None => mv(new, false),
                    Some(p) => {
                        if p.color != c {
                            mv(new, true);
                        }
                        break;
                    }
                }
            }
            new = old;

            while new.southwest(1).is_some() {
                new = new.southwest(1).unwrap();
                match self.piece(new) {
                    None => mv(new, false),
                    Some(p) => {
                        if p.color != c {
                            mv(new, true);
                        }
                        break;
                    }
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
            let mut mv = |new, takes| {
                moves.push(Move {
                    to: new,
                    takes: takes,
                    ..m
                })
            };

            while new.north(1).is_some() {
                new = new.north(1).unwrap();
                match self.piece(new) {
                    None => mv(new, false),
                    Some(p) => {
                        if p.color != c {
                            mv(new, true);
                        }
                        break;
                    }
                }
            }
            new = old;

            while new.south(1).is_some() {
                new = new.south(1).unwrap();
                match self.piece(new) {
                    None => mv(new, false),
                    Some(p) => {
                        if p.color != c {
                            mv(new, true);
                        }
                        break;
                    }
                }
            }
            new = old;

            while new.east(1).is_some() {
                new = new.east(1).unwrap();
                match self.piece(new) {
                    None => mv(new, false),
                    Some(p) => {
                        if p.color != c {
                            mv(new, true);
                        }
                        break;
                    }
                }
            }
            new = old;

            while new.west(1).is_some() {
                new = new.west(1).unwrap();
                match self.piece(new) {
                    None => mv(new, false),
                    Some(p) => {
                        if p.color != c {
                            mv(new, true);
                        }
                        break;
                    }
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
            let mut mv = |new, takes| {
                moves.push(Move {
                    to: new,
                    takes: takes,
                    ..m
                })
            };

            while new.northeast(1).is_some() {
                new = new.northeast(1).unwrap();
                match self.piece(new) {
                    None => mv(new, false),
                    Some(p) => {
                        if p.color != c {
                            mv(new, true);
                        }
                        break;
                    }
                }
            }
            new = old;

            while new.northwest(1).is_some() {
                new = new.northwest(1).unwrap();
                match self.piece(new) {
                    None => mv(new, false),
                    Some(p) => {
                        if p.color != c {
                            mv(new, true);
                        }
                        break;
                    }
                }
            }
            new = old;

            while new.southeast(1).is_some() {
                new = new.southeast(1).unwrap();
                match self.piece(new) {
                    None => mv(new, false),
                    Some(p) => {
                        if p.color != c {
                            mv(new, true);
                        }
                        break;
                    }
                }
            }
            new = old;

            while new.southwest(1).is_some() {
                new = new.southwest(1).unwrap();
                match self.piece(new) {
                    None => mv(new, false),
                    Some(p) => {
                        if p.color != c {
                            mv(new, true);
                        }
                        break;
                    }
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
                old.mv(vert, horiz).map(|new| match self.piece(new) {
                    None => moves.push(Move {
                        to: new,
                        takes: false,
                        ..m
                    }),
                    Some(p) => {
                        if p.color != c {
                            moves.push(Move {
                                to: new,
                                takes: true,
                                ..m
                            })
                        }
                    }
                });
            };
            mv(1, 2);
            mv(1, -2);
            mv(-1, -2);
            mv(-1, 2);
            mv(2, 1);
            mv(2, -1);
            mv(-2, -1);
            mv(-2, 1);
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
        let king_moves = [
            (1, 0),
            (1, 1),
            (1, -1),
            (-1, 0),
            (-1, 1),
            (-1, -1),
            (0, 1),
            (0, -1),
        ];
        for &(vert, horiz) in king_moves.iter() {
            old.mv(vert, horiz).map(|new| match self.piece(new) {
                None => moves.push(Move {
                    to: new,
                    takes: false,
                    ..m
                }),
                Some(p) => {
                    if p.color != c {
                        moves.push(Move {
                            to: new,
                            takes: true,
                            ..m
                        })
                    }
                }
            });
        }

        let castle = Move {
            kind: PieceType::King,
            from: Pos::zero(),
            to: Pos::zero(),
            takes: false,
            en_passant: false,
            promotion: None,
            castle: None,
        };

        if self.castle_kingside_rights(c)
            && !self.occupied(
                old.east(1)
                    .expect("[Board::king_moves] confusing castling rights!"),
            )
            && !self.occupied(
                old.east(2)
                    .expect("[Board::king_moves] confusing castling rights!"),
            )
            && !self.color_threatens(c.other(), old)
            && !self.color_threatens(c.other(), old.east(1).unwrap())
        {
            moves.push(Move {
                castle: Some(Castle::Kingside),
                ..castle
            });
        }

        if self.castle_queenside_rights(c)
            && !self.occupied(
                old.west(1)
                    .expect("[Board::king_moves] confusing castling rights!"),
            )
            && !self.occupied(
                old.west(2)
                    .expect("[Board::king_moves] confusing castling rights!"),
            )
            && !self.occupied(
                old.west(3)
                    .expect("[Board::king_moves] confusing castling rights!"),
            )
            && !self.color_threatens(c.other(), old)
            && !self.color_threatens(c.other(), old.west(1).unwrap())
        {
            moves.push(Move {
                castle: Some(Castle::Queenside),
                ..castle
            });
        }
        moves
    }

    // checks for legality
    pub fn make_move(&self, mv: &Move) -> Result<Board, ChessError> {
        let color = self.color_to_move;
        let mut b = Board {
            color_to_move: color.other(),
            ..*self
        };
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
                            if self.color_threatens(color.other(), pos!("e1"))
                                || self.color_threatens(color.other(), pos!("f1"))
                                || self.color_threatens(color.other(), pos!("g1"))
                            {
                                illegal_move_error!(
                                    "[make_move] {}: white cannot castle kingside through check!",
                                    mv
                                );
                            }
                            if self.occupied(pos!("f1")) || self.occupied(pos!("g1")) {
                                illegal_move_error!("[make_move] {}: white cannot castle kingside: spaces occupied!", mv);
                            }
                            let k = match b.get_piece_at("e1") {
                                Some(
                                    p
                                    @
                                    Piece {
                                        kind: PieceType::King,
                                        color: Color::White,
                                    },
                                ) => p,
                                _ => {
                                    illegal_move_error!("[make_move] {}: no white king at e1!", mv)
                                }
                            };
                            let r = match b.get_piece_at("h1") {
                                Some(
                                    p
                                    @
                                    Piece {
                                        kind: PieceType::Rook,
                                        color: Color::White,
                                    },
                                ) => p,
                                _ => {
                                    illegal_move_error!("[make_move] {}: no white rook at h1!", mv)
                                }
                            };
                            b.put_piece_at(k, "g1");
                            b.put_piece_at(r, "f1");
                            b.castle_rights[0] = false;
                            b.castle_rights[1] = false;
                        }
                        Color::Black => {
                            if self.color_threatens(color.other(), pos!("e8"))
                                || self.color_threatens(color.other(), pos!("f8"))
                                || self.color_threatens(color.other(), pos!("g8"))
                            {
                                illegal_move_error!(
                                    "[make_move] {} black cannot castle kingside through check!",
                                    mv
                                );
                            }
                            if self.occupied(pos!("f8")) || self.occupied(pos!("g8")) {
                                illegal_move_error!("[make_move] {}: black cannot castle kingside: spaces occupied!", mv);
                            }
                            let k = match b.get_piece_at("e8") {
                                Some(
                                    p
                                    @
                                    Piece {
                                        kind: PieceType::King,
                                        color: Color::Black,
                                    },
                                ) => p,
                                _ => {
                                    illegal_move_error!("[make_move] {}: no black king at e8!", mv)
                                }
                            };
                            let r = match b.get_piece_at("h8") {
                                Some(
                                    p
                                    @
                                    Piece {
                                        kind: PieceType::Rook,
                                        color: Color::Black,
                                    },
                                ) => p,
                                _ => {
                                    illegal_move_error!("[make_move] {}: no black rook at h8!", mv)
                                }
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
                            if self.color_threatens(color.other(), pos!("e1"))
                                || self.color_threatens(color.other(), pos!("d1"))
                                || self.color_threatens(color.other(), pos!("c1"))
                            {
                                illegal_move_error!(
                                    "[make_move] {}: white cannot castle queenside through check!",
                                    mv
                                );
                            }
                            if self.occupied(pos!("b1"))
                                || self.occupied(pos!("c1"))
                                || self.occupied(pos!("d1"))
                            {
                                illegal_move_error!("[make_move] {}: white cannot castle queenside: spaces occupied!", mv);
                            }
                            let k = match b.get_piece_at("e1") {
                                Some(
                                    p
                                    @
                                    Piece {
                                        kind: PieceType::King,
                                        color: Color::White,
                                    },
                                ) => p,
                                _ => {
                                    illegal_move_error!("[make_move] {}: no white king at e1!", mv)
                                }
                            };
                            let r = match b.get_piece_at("a1") {
                                Some(
                                    p
                                    @
                                    Piece {
                                        kind: PieceType::Rook,
                                        color: Color::White,
                                    },
                                ) => p,
                                _ => {
                                    illegal_move_error!("[make_move] {}: no white rook at a1!", mv)
                                }
                            };
                            b.put_piece_at(k, "c1");
                            b.put_piece_at(r, "d1");
                            b.castle_rights[0] = false;
                            b.castle_rights[1] = false;
                        }
                        Color::Black => {
                            if self.color_threatens(color.other(), pos!("e8"))
                                || self.color_threatens(color.other(), pos!("d8"))
                                || self.color_threatens(color.other(), pos!("c8"))
                            {
                                illegal_move_error!(
                                    "[make_move] {}: black cannot castle queenside through check!",
                                    mv
                                );
                            }
                            if self.occupied(pos!("b8"))
                                || self.occupied(pos!("c8"))
                                || self.occupied(pos!("d8"))
                            {
                                illegal_move_error!("[make_move] {}: black cannot castle queenside: spaces occupied!", mv);
                            }
                            let k = match b.get_piece_at("e8") {
                                Some(
                                    p
                                    @
                                    Piece {
                                        kind: PieceType::King,
                                        color: Color::Black,
                                    },
                                ) => p,
                                _ => {
                                    illegal_move_error!("[make_move] {}: no black king at e8!", mv)
                                }
                            };
                            let r = match b.get_piece_at("a8") {
                                Some(
                                    p
                                    @
                                    Piece {
                                        kind: PieceType::Rook,
                                        color: Color::Black,
                                    },
                                ) => p,
                                _ => {
                                    illegal_move_error!("[make_move] {}: no black rook at a8!", mv)
                                }
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
            let p = match b.board[mv.from.index()].take() {
                Some(p) => p,
                None => illegal_move_error!("[make_move] {}: no piece at {}", mv, mv.from),
            };
            // check that we are moving a white piece if it is white's turn
            if p.color != color {
                illegal_move_error!(
                    "[make_move] {}: tried to move {}'s piece but it is {}'s turn!",
                    mv,
                    p.color,
                    color
                );
            }
            // find the position of the piece we are capturing
            let target_piece_at = match color {
                Color::White => mv.to.south(1).unwrap(),
                Color::Black => mv.to.north(1).unwrap(),
            };
            // there should be no peice at the en passant target
            match b.board[mv.to.index()] {
                Some(p) => board_state_error!(
                    "[make_move] {}: there should be no piece at {} but I found {}",
                    mv,
                    mv.to,
                    p
                ),
                None => {}
            }
            // remove the target piece from the board
            let q = b.board[target_piece_at.index()].take();
            // check that there was actually something there
            if q.is_none() {
                illegal_move_error!(
                    "[make_move] {}: there was no piece at {}!",
                    mv,
                    target_piece_at
                );
            }
            // check that we're taking a piece of the opposite color!
            if q.map_or(false, |q| q.color == color) {
                illegal_move_error!("[make_move] {}: taking a piece of the same color!", mv);
            }
            // place the capturing piece
            b.board[mv.to.index()] = Some(p);
            // reset en passant target
            b.en_passant_target = None;
        } else {
            // grab the moving/capturing piece
            let p = b.board[mv.from.index()]
                .take()
                .expect(&format!("[make_move] {}: no piece at {}", mv, mv.from));
            // check that it is the right color
            if p.color != color {
                illegal_move_error!(
                    "[make_move] {}: tried to move {}'s piece but it is {}'s turn!",
                    mv,
                    p.color,
                    color
                );
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
                        if mv.from == pos!("h1") {
                            b.castle_rights[0] = false
                        }
                        if mv.from == pos!("a1") {
                            b.castle_rights[1] = false
                        }
                    }
                    Color::Black => {
                        if mv.from == pos!("h8") {
                            b.castle_rights[2] = false
                        }
                        if mv.from == pos!("a8") {
                            b.castle_rights[3] = false
                        }
                    }
                }
            }
            // grab the potentially nonexistant target piece
            let q = b.board[mv.to.index()].take();
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
                b.board[mv.to.index()] = Some(Piece { kind: prom, ..p });
            } else {
                // place moving/capturing piece
                b.board[mv.to.index()] = Some(p);
            }
        }
        let kings = b.get_pieces_by_type_and_color(PieceType::King, color);
        if kings.len() == 1 && b.color_threatens(color.other(), kings[0]) {
            illegal_move_error!("moving into check");
        }
        Ok(b)
    }
}
