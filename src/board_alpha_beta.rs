use board::Board;
use moves::Move;
use piece::{Color, PieceType};
use rand::{self, Rng};
use util::ChessError;

use std::cmp::{min, max};
use std::sync::Arc;
use std::sync::Mutex;

impl Board {
    pub fn random_move(&self) -> Result<(Move, isize), ChessError> {
        let mut rng = rand::thread_rng();
        let ms = self.legal_moves()?;
        let i = rng.gen::<usize>() % ms.len();
        Ok((ms[i], 0))
    }

    // get score of board in centipawns
    pub fn score(&self, color: Color) -> isize {
        let mut score = 0;
        // let color = self.color_to_move;
        for (pos, piece) in self.get_pieces_by_color(color) {
            score += pos.value();
            match piece.kind {
                PieceType::Pawn   => score += 100,
                PieceType::Knight => score += 300,
                PieceType::Bishop => score += 300,
                PieceType::Rook   => score += 500,
                PieceType::Queen  => score += 900,
                PieceType::King   => score += isize::max_value()/2,
            }
        }
        for (pos, piece) in self.get_pieces_by_color(color.other()) {
            score -= pos.value();
            match piece.kind {
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
        let mut rng = rand::thread_rng();
        let mut best_score = isize::min_value();
        let mut best_move = None;
        let moves = self.legal_moves()?;
        for mv in moves {
            let score;
            if depth == 0 {
                score = match self.make_move(&mv) {
                    Err(ChessError::Checkmate) => return Ok((mv, isize::max_value()/2)),
                    Err(ChessError::Stalemate) => 0,
                    Err(e)                     => return Err(e),
                    Ok(new_board)              => new_board.score(self.color_to_move),
                };
            } else {
                score = self.make_move(&mv)?.alpha_beta(depth, Arc::new(Mutex::new(false)));
            }
            if score > best_score || (score == best_score && rng.gen())
            {
                best_move = Some(mv);
                best_score = score;
            }
        }
        Ok((best_move.unwrap(), best_score))
    }

    pub fn alpha_beta(&self, depth: usize, abort: Arc<Mutex<bool>>) -> isize {
        self.alpha_beta_rec(self.color_to_move.other(), depth, isize::min_value(), isize::max_value(), abort)
    }

    fn alpha_beta_rec(&self, my_color: Color, depth: usize, alpha_in: isize, beta_in: isize,
                      abort: Arc<Mutex<bool>>)
        -> isize
    {
        let mut alpha = alpha_in;
        let mut beta  = beta_in;
        if depth == 0 || *abort.lock().unwrap() {
            return self.score(my_color);
        }
        if self.color_to_move == my_color {
            // maximizing player
            let mut v = isize::min_value();
            match self.legal_moves() {
                Err(ChessError::Checkmate) => return isize::min_value()/2,
                Err(ChessError::Stalemate) => return 0,
                Err(e) => panic!("{}", e),
                Ok(moves) => for mv in moves {
                    let score = self.make_move(&mv).unwrap().alpha_beta_rec(my_color, depth - 1, alpha, beta, abort.clone());
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
                Err(ChessError::Checkmate) => return isize::max_value()/2,
                Err(ChessError::Stalemate) => return 0,
                Err(e) => panic!("{}", e),
                Ok(moves) => for mv in moves {
                    let score = self.make_move(&mv).unwrap().alpha_beta_rec(my_color, depth - 1, alpha, beta, abort.clone());
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

}
