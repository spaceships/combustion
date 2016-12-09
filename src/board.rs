use util::ChessError;
use std::cmp::{min, max};
use rand::{self, Rng};
use position::Pos;
use piece::{Color, PieceType, Piece};
use moves::{Castle, Move};

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
    fn get_piece_at(&mut self, s: &str) -> Option<Piece> {
        self.board[pos!(s).index()].take()
    }

    // ignores bad formating of the string!
    fn put_piece_at(&mut self, p: Piece, s: &str) {
        let pos = Pos::from_algebra(s).unwrap();
        assert!(self.piece(pos).is_none());
        self.board[pos.index()] = Some(p);
    }

    fn pieces(&self, f: &Fn(Piece) -> bool) -> Vec<(Pos, Piece)> {
        let mut res = Vec::new();
        for ix in 0..64 {
            self.board[ix].map(|p| {
                if f(p) {
                    res.push((Pos::from_index(ix),p));
                }
            });
        }
        res
    }

    pub fn get_pieces_by_type_and_color(&self, k: PieceType, c: Color) -> Vec<Pos> {
        let q = Piece { kind: k, color: c };
        self.pieces(&|p| p == q).into_iter().map(|(pos,_)| pos).collect()
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

    pub fn threatens(&self, c: Color, old: Pos) -> bool {
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

    // checks for legality
    pub fn make_move(&self, mv: &Move) -> Result<Board, ChessError> {
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
                            if self.threatens(color.other(), pos!("e1")) ||
                               self.threatens(color.other(), pos!("f1")) ||
                               self.threatens(color.other(), pos!("g1")) {
                                illegal_move_error!("[make_move] {}: white cannot castle kingside through check!", mv);
                            }
                            if self.occupied(pos!("f1")) || self.occupied(pos!("g1")) {
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
                            if self.threatens(color.other(), pos!("e8")) ||
                               self.threatens(color.other(), pos!("f8")) ||
                               self.threatens(color.other(), pos!("g8")) {
                                illegal_move_error!("[make_move] {} black cannot castle kingside through check!", mv);
                            }
                            if self.occupied(pos!("f8")) || self.occupied(pos!("g8")) {
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
                            if self.threatens(color.other(), pos!("e1")) ||
                               self.threatens(color.other(), pos!("d1")) ||
                               self.threatens(color.other(), pos!("c1")) {
                                illegal_move_error!("[make_move] {}: white cannot castle queenside through check!", mv);
                            }
                            if self.occupied(pos!("b1")) ||
                               self.occupied(pos!("c1")) ||
                               self.occupied(pos!("d1")) {
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
                            if self.threatens(color.other(), pos!("e8")) ||
                               self.threatens(color.other(), pos!("d8")) ||
                               self.threatens(color.other(), pos!("c8")) {
                                illegal_move_error!("[make_move] {}: black cannot castle queenside through check!", mv);
                            }
                            if self.occupied(pos!("b8")) ||
                               self.occupied(pos!("c8")) ||
                               self.occupied(pos!("d8")) {
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
            let p = match b.board[mv.from.index()].take() {
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
            match b.board[mv.to.index()] {
                Some(p) => board_state_error!("[make_move] {}: there should be no piece at {} but I found {}", mv, mv.to, p),
                None => {}
            }
            // remove the target piece from the board
            let q = b.board[target_piece_at.index()].take();
            // check that there was actually something there
            if q.is_none() {
                illegal_move_error!("[make_move] {}: there was no piece at {}!", mv, target_piece_at);
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
            let p = b.board[mv.from.index()].take().expect(&format!("[make_move] {}: no piece at {}", mv, mv.from));
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
                        if mv.from == pos!("h1") { b.castle_rights[0] = false }
                        if mv.from == pos!("a1") { b.castle_rights[1] = false }
                    }
                    Color::Black => {
                        if mv.from == pos!("h8") { b.castle_rights[2] = false }
                        if mv.from == pos!("a8") { b.castle_rights[3] = false }
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
                b.board[mv.to.index()] = Some(Piece{ kind: prom, .. p});
            } else {
                // place moving/capturing piece
                b.board[mv.to.index()] = Some(p);
            }
        }
        let kings = b.get_pieces_by_type_and_color(PieceType::King, color);
        if kings.len() == 1 && b.threatens(color.other(), kings[0]) {
            illegal_move_error!("moving into check");
        }
        Ok(b)
    }

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
                PieceType::King   => score += isize::max_value()/2,
            }
        }
        for (_, p) in self.get_pieces_by_color(color.other()) {
            match p.kind {
                PieceType::Pawn   => score -= 100,
                PieceType::Knight => score -= 300,
                PieceType::Bishop => score -= 300,
                PieceType::Rook   => score -= 500,
                PieceType::Queen  => score -= 900,
                PieceType::King   => score -= isize::max_value()/2,
            }
        }
        score
    }

    pub fn best_move(&self, depth: usize) -> Result<(Move, isize), ChessError> {
        // find the move with the weakest response
        let moves = self.legal_moves()?;
        let mut rng = rand::thread_rng();
        let mut best_score = isize::min_value();
        let mut best_move = None;
        for mv in moves {
            let score;
            if depth == 0 {
                score = match self.make_move(&mv) {
                    Err(ChessError::Checkmate) => return Ok((mv, isize::max_value())),
                    Err(ChessError::Stalemate) => 0,
                    Err(e)                     => return Err(e),
                    Ok(new_board)              => new_board.score(self.color_to_move),
                };
            } else {
                score = self.make_move(&mv)?.alpha_beta(
                    self.color_to_move, depth, isize::min_value(), isize::max_value());
            }
            if score > best_score || (score == best_score && rng.gen())
            {
                best_move = Some(mv);
                best_score = score;
            }
        }
        Ok((best_move.unwrap(), best_score))
    }

    fn alpha_beta(&self, my_color: Color, depth: usize, alpha_in: isize, beta_in: isize) -> isize
    {
        let mut alpha = alpha_in;
        let mut beta  = beta_in;
        if depth == 0 {
            return self.score(my_color);
        }
        if self.color_to_move == my_color {
            // maximizing player
            let mut v = isize::min_value();
            match self.legal_moves() {
                Err(ChessError::Checkmate) => return isize::min_value(),
                Err(ChessError::Stalemate) => return 0,
                Err(e) => panic!("{}", e),
                Ok(moves) => for mv in moves {
                    let score = self.make_move(&mv).unwrap().alpha_beta(my_color, depth - 1, alpha, beta);
                    v     = max(v, score);
                    alpha = max(alpha, v);
                    if beta <= alpha {
                        break;
                    }
                },
            }
            v
        } else {
            let mut v = isize::max_value();
            match self.legal_moves() {
                Err(ChessError::Checkmate) => return isize::max_value(),
                Err(ChessError::Stalemate) => return 0,
                Err(e) => panic!("{}", e),
                Ok(moves) => for mv in moves {
                    let score = self.make_move(&mv).unwrap().alpha_beta(my_color, depth - 1, alpha, beta);
                    v = min(v, score);
                    beta = min(beta, v);
                    if beta <= alpha {
                        break;
                    }
                },
            }
            v
        }
    }

    pub fn random_move(&self) -> Result<(Move, isize), ChessError> {
        let mut rng = rand::thread_rng();
        let ms = self.legal_moves()?;
        let i = rng.gen::<usize>() % ms.len();
        Ok((ms[i], 0))
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
