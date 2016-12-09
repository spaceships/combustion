use board::Board;
use piece::{PieceType, Piece, Color};
use position::Pos;
use util::ChessError;

impl Board {
    pub fn from_fen(fen: &str) -> Result<Board, ChessError> {
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
}
