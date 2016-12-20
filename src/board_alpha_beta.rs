use board::Board;
use moves::Move;
use piece::{Piece, Color, PieceType};
use rand::{self, Rng};
use util::ChessError;
use position::Pos;
use transposition_table::TranspositionTable;

use std::cmp::{min, max};
use std::sync::Arc;
use std::sync::RwLock;

impl Board {
    pub fn random_move(&self) -> Result<(Move, isize), ChessError> {
        let mut rng = rand::thread_rng();
        let ms = self.legal_moves()?;
        let i = rng.gen::<usize>() % ms.len();
        Ok((ms[i], 0))
    }

    fn piece_score(&self, pos: Pos, piece: Piece) -> isize {
        let mut score = 0;
        score += self.nthreats(pos, piece);
        score += pos.value();
        match piece.kind {
            PieceType::Pawn   => score += 100,
            PieceType::Knight => score += 300,
            PieceType::Bishop => score += 300,
            PieceType::Rook   => score += 500,
            PieceType::Queen  => score += 900,
            PieceType::King   => score += isize::max_value()/2,
        }
        score
    }

    // get score of board in centipawns
    pub fn score(&self, color: Color) -> isize {
        let mut score = 0;
        for (pos, piece) in self.get_pieces_by_color(color) {
            score += self.piece_score(pos, piece);
        }
        for (pos, piece) in self.get_pieces_by_color(color.other()) {
            score -= self.piece_score(pos, piece);
        }
        score
    }

    // find the move with the weakest response - single threaded
    pub fn best_move(&self, max_depth: usize) -> Result<(Move, isize), ChessError> {
        let mut rng = rand::thread_rng();
        let mut best_score = isize::min_value();
        let mut best_move = None;
        let moves = self.legal_moves()?;
        for mv in moves {
            let score;
            if max_depth == 0 {
                score = match self.make_move(&mv) {
                    Err(ChessError::Checkmate) => return Ok((mv, isize::max_value()-1)),
                    Err(ChessError::Stalemate) => 0,
                    Err(e)                     => return Err(e),
                    Ok(new_board)              => new_board.score(self.color_to_move),
                };
            } else {
                let tt = Arc::new(TranspositionTable::new(max_depth+1));
                score = self.make_move(&mv)?.alpha_beta(max_depth, None, Some(tt.clone()));
            }
            if score > best_score || (score == best_score && rng.gen())
            {
                best_move = Some(mv);
                best_score = score;
            }
        }
        Ok((best_move.unwrap(), best_score))
    }

    pub fn alpha_beta(&self, max_depth: usize,
                      abort: Option<Arc<RwLock<bool>>>,
                      transposition_table: Option<Arc<TranspositionTable>>)
        -> isize
    {
        self.alpha_beta_rec(self.color_to_move.other(), 0, max_depth,
                            isize::min_value(), isize::max_value(),
                            &abort,
                            &transposition_table)
    }

    fn alpha_beta_rec(&self, my_color: Color,
                      depth: usize, max_depth: usize,
                      alpha_in: isize, beta_in: isize,
                      abort: &Option<Arc<RwLock<bool>>>,
                      tt: &Option<Arc<TranspositionTable>>)
        -> isize
    {
        // if the transposition table includes this board state at this depth,
        // return the previous value
        if let Some(ref table) = *tt {
            if let Some(result) = table.get(self, depth) {
                return result;
            }
        }

        let mut alpha = alpha_in;
        let mut beta  = beta_in;
        if depth == max_depth || abort.as_ref().map_or(false, |mutex| *mutex.read().unwrap()) {
            return self.score(my_color);
        }

        let ret;
        if self.color_to_move == my_color {
            // maximizing player
            let mut v = isize::min_value();
            match self.legal_moves() {
                Err(ChessError::Checkmate) => return isize::min_value()+1,
                Err(ChessError::Stalemate) => return 0,
                Err(e) => panic!("{}", e),
                Ok(moves) => for mv in moves {
                    let b = self.make_move(&mv).unwrap();
                    let score = b.alpha_beta_rec(my_color, depth + 1, max_depth, alpha, beta, abort, tt);
                    v     = max(v, score);
                    alpha = max(alpha, v);
                    if beta <= alpha { break }
                },
            }
            ret = v;
        }

        else {
            let mut v = isize::max_value();
            match self.legal_moves() {
                Err(ChessError::Checkmate) => return isize::max_value()-1,
                Err(ChessError::Stalemate) => return 0,
                Err(e) => panic!("{}", e),
                Ok(moves) => for mv in moves {
                    let b = self.make_move(&mv).unwrap();
                    let score = b.alpha_beta_rec(my_color, depth + 1, max_depth, alpha, beta, abort, tt);
                    v = min(v, score);
                    beta = min(beta, v);
                    if beta <= alpha { break }
                },
            }
            ret = v;
        }

        // update the transposition table with the result
        if let Some(ref table) = *tt {
            table.insert(self, depth, ret);
        }

        ret
    }

}
