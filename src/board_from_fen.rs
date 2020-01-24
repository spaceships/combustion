use crate::board::Board;
use crate::piece::{Color, Piece, PieceType};
use crate::position::Pos;
use crate::util::ChessError;

impl Board {
    pub fn to_fen(&self) -> String {
        let mut s = String::new();
        for i in 0..8 {
            let mut n = 0;
            for j in 0..8 {
                match self.piece(Pos::new(i, j)) {
                    None => {
                        n += 1;
                        if j == 7 {
                            s.push_str(&format!("{}", n));
                        }
                    }

                    Some(p) => {
                        if n > 0 {
                            s.push_str(&format!("{}", n));
                            n = 0;
                        }
                        let mut c;
                        match p.kind {
                            PieceType::Pawn => c = 'p',
                            PieceType::Knight => c = 'n',
                            PieceType::Bishop => c = 'b',
                            PieceType::Rook => c = 'r',
                            PieceType::Queen => c = 'q',
                            PieceType::King => c = 'k',
                        }
                        if p.color == Color::White {
                            c = c.to_uppercase().collect::<Vec<char>>()[0];
                        }
                        s.push(c);
                    }
                }
            }
            if i < 7 {
                s.push('/');
            } else {
                s.push(' ');
            }
        }

        match self.color_to_move {
            Color::White => s.push_str("w "),
            Color::Black => s.push_str("b "),
        }

        if !self.castle_rights.iter().any(|&x| x) {
            s.push('-');
        } else {
            if self.castle_rights[0] {
                s.push('K');
            }
            if self.castle_rights[1] {
                s.push('Q');
            }
            if self.castle_rights[2] {
                s.push('k');
            }
            if self.castle_rights[3] {
                s.push('q');
            }
        }

        s.push(' ');

        match self.en_passant_target {
            Some(ep) => s.push_str(&ep.to_algebra()),
            None => s.push('-'),
        }

        s.push_str(&format!(" {} {}", self.halfmove_clock, self.move_number));

        s
    }

    pub fn from_fen(fen: &str) -> Result<Board, ChessError> {
        let mut b = Board::new();
        let mut i = 0;
        let mut j = 0;
        let tokens: Vec<&str> = fen.split(" ").collect();

        let check = |i, j| {
            if i * 8 + j >= 64 {
                Err(ChessError::ParseError(format!(
                    "[from_fen] index out of bounds i={} j={}!",
                    i, j
                )))
            } else {
                Ok(())
            }
        };

        // parse board
        for c in tokens[0].chars() {
            match c {
                ' ' => break,
                '/' => {
                    i += 1;
                    j = 0;
                }

                n @ '1'..='8' => {
                    j += n
                        .to_string()
                        .parse::<usize>()
                        .expect("couldn't read number!");
                }

                'P' => {
                    check(i, j)?;
                    b.board[i * 8 + j] = Some(Piece {
                        kind: PieceType::Pawn,
                        color: Color::White,
                    });
                    j += 1;
                }
                'p' => {
                    check(i, j)?;
                    b.board[i * 8 + j] = Some(Piece {
                        kind: PieceType::Pawn,
                        color: Color::Black,
                    });
                    j += 1;
                }
                'B' => {
                    check(i, j)?;
                    b.board[i * 8 + j] = Some(Piece {
                        kind: PieceType::Bishop,
                        color: Color::White,
                    });
                    j += 1;
                }
                'b' => {
                    check(i, j)?;
                    b.board[i * 8 + j] = Some(Piece {
                        kind: PieceType::Bishop,
                        color: Color::Black,
                    });
                    j += 1;
                }
                'N' => {
                    check(i, j)?;
                    b.board[i * 8 + j] = Some(Piece {
                        kind: PieceType::Knight,
                        color: Color::White,
                    });
                    j += 1;
                }
                'n' => {
                    check(i, j)?;
                    b.board[i * 8 + j] = Some(Piece {
                        kind: PieceType::Knight,
                        color: Color::Black,
                    });
                    j += 1;
                }
                'R' => {
                    check(i, j)?;
                    b.board[i * 8 + j] = Some(Piece {
                        kind: PieceType::Rook,
                        color: Color::White,
                    });
                    j += 1;
                }
                'r' => {
                    check(i, j)?;
                    b.board[i * 8 + j] = Some(Piece {
                        kind: PieceType::Rook,
                        color: Color::Black,
                    });
                    j += 1;
                }
                'Q' => {
                    check(i, j)?;
                    b.board[i * 8 + j] = Some(Piece {
                        kind: PieceType::Queen,
                        color: Color::White,
                    });
                    j += 1;
                }
                'q' => {
                    check(i, j)?;
                    b.board[i * 8 + j] = Some(Piece {
                        kind: PieceType::Queen,
                        color: Color::Black,
                    });
                    j += 1;
                }
                'K' => {
                    check(i, j)?;
                    b.board[i * 8 + j] = Some(Piece {
                        kind: PieceType::King,
                        color: Color::White,
                    });
                    j += 1;
                }
                'k' => {
                    check(i, j)?;
                    b.board[i * 8 + j] = Some(Piece {
                        kind: PieceType::King,
                        color: Color::Black,
                    });
                    j += 1;
                }

                c => parse_error!("[from_fen] unexpected \"{}\"", c),
            }
        }

        // parse turn
        match tokens[1] {
            "w" | "W" => b.color_to_move = Color::White,
            "b" | "B" => b.color_to_move = Color::Black,
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
            s => b.en_passant_target = Some(Pos::from_algebra(s)?),
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
}

#[cfg(test)]
mod tests {
    use crate::board::Board;

    #[test]
    fn fen_correct() {
        let p = "1K6/2P5/1p3P2/1k2P3/1qnP1B2/3Q4/8/8 b - - 0 1";
        let q = &Board::from_fen(p).unwrap().to_fen();
        println!("\np={}\nq={}", p, q);
        assert_eq!(p, q);

        let p = "4k2r/8/5Q2/8/8/8/8/8 b kq - 0 1";
        println!("\n{}", Board::from_fen(p).unwrap());
        let q = &Board::from_fen(p).unwrap().to_fen();
        println!("\np={}\nq={}", p, q);
        assert_eq!(p, q);

        let p = "r3k2r/8/8/8/8/8/8/8 b kq - 0 1";
        let q = &Board::from_fen(p).unwrap().to_fen();
        println!("\np={}\nq={}", p, q);
        assert_eq!(p, q);
    }
}
