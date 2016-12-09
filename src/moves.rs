use std::fmt;

use piece::{Color, PieceType};
use position::Pos;
use util::ChessError;
use board::Board;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Castle {
    Kingside,
    Queenside,
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
pub struct Move {
    pub kind: PieceType,
    pub from: Pos,
    pub to: Pos,
    pub takes: bool,
    pub en_passant: bool,
    pub promotion: Option<PieceType>,
    pub castle: Option<Castle>,
}

impl Move {
    #[allow(dead_code)]
    pub fn from_algebra(s: &str) -> Result<Move, ChessError> {
        println!("{}", s);
        if s == "O-O" {
            Ok( Move {
                kind: PieceType::King,
                from: Pos::zero(), to: Pos::zero(), takes: false,
                en_passant: false, promotion: None,
                castle: Some(Castle::Kingside),
            })
        } else if s == "O-O-O" {
            Ok( Move {
                kind: PieceType::King,
                from: Pos::zero(), to: Pos::zero(), takes: false,
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
                from: Pos::from_algebra(&from)?,
                to:   Pos::from_algebra(&to)?,
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
            if from == pos!("e1") && to == pos!("g1") ||
               from == pos!("e8") && to == pos!("g8")
            {
                Some(Castle::Kingside)
            }

            else if from == pos!("e1") && to == pos!("c1") ||
                    from == pos!("e8") && to == pos!("c8")
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

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.castle {
            Some(Castle::Kingside)  => write!(f, "O-O"),
            Some(Castle::Queenside) => write!(f, "O-O-O"),
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
