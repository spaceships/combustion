use board::Board;
use moves::Move;
use piece::{Color, PieceType};
use rand::{self, Rng};
use util::ChessError;

use std::cmp::{min, max};

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
        let mut rng = rand::thread_rng();
        let mut best_score = isize::min_value();
        let mut best_move = None;
        let moves = self.legal_moves()?;
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

}
