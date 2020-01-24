use crate::board::Board;
use crate::piece::{Color, Piece, PieceType};
use crate::position::Pos;

use std::cell::RefCell;

impl Board {
    pub fn color_threatens(&self, c: Color, old: Pos) -> bool {
        for (new, piece) in self.get_pieces_by_color(c) {
            match piece.kind {
                PieceType::Pawn => {
                    match c {
                        Color::White => {
                            // omg en passant is complicated
                            if new.northeast(1).map_or(false, |ray| {
                                ray == old
                                    || self.is_en_passant_target(ray)
                                        && ray.south(1).unwrap() == old
                            }) || new.northwest(1).map_or(false, |ray| {
                                ray == old
                                    || self.is_en_passant_target(ray)
                                        && ray.south(1).unwrap() == old
                            }) {
                                return true;
                            }
                        }
                        Color::Black => {
                            if new.southeast(1).map_or(false, |ray| {
                                ray == old
                                    || self.is_en_passant_target(ray)
                                        && ray.north(1).unwrap() == old
                            }) || new.southwest(1).map_or(false, |ray| {
                                ray == old
                                    || self.is_en_passant_target(ray)
                                        && ray.north(1).unwrap() == old
                            }) {
                                return true;
                            }
                        }
                    }
                }

                PieceType::King => {
                    if new.mv(1, 1).map_or(false, |ray| ray == old)
                        || new.mv(1, 0).map_or(false, |ray| ray == old)
                        || new.mv(1, -1).map_or(false, |ray| ray == old)
                        || new.mv(0, 1).map_or(false, |ray| ray == old)
                        || new.mv(0, -1).map_or(false, |ray| ray == old)
                        || new.mv(-1, 1).map_or(false, |ray| ray == old)
                        || new.mv(-1, 0).map_or(false, |ray| ray == old)
                        || new.mv(-1, -1).map_or(false, |ray| ray == old)
                    {
                        return true;
                    }
                }

                PieceType::Knight => {
                    if new.mv(1, 2).map_or(false, |ray| ray == old)
                        || new.mv(1, -2).map_or(false, |ray| ray == old)
                        || new.mv(-1, -2).map_or(false, |ray| ray == old)
                        || new.mv(-1, 2).map_or(false, |ray| ray == old)
                        || new.mv(2, 1).map_or(false, |ray| ray == old)
                        || new.mv(2, -1).map_or(false, |ray| ray == old)
                        || new.mv(-2, -1).map_or(false, |ray| ray == old)
                        || new.mv(-2, 1).map_or(false, |ray| ray == old)
                    {
                        return true;
                    }
                }

                PieceType::Rook => {
                    let mut ray = new;
                    while ray.north(1).is_some() {
                        ray = ray.north(1).unwrap();
                        if ray == old {
                            return true;
                        }
                        if self.piece(ray).is_some() {
                            break;
                        }
                    }

                    ray = new;
                    while ray.south(1).is_some() {
                        ray = ray.south(1).unwrap();
                        if ray == old {
                            return true;
                        }
                        if self.piece(ray).is_some() {
                            break;
                        }
                    }

                    ray = new;
                    while ray.east(1).is_some() {
                        ray = ray.east(1).unwrap();
                        if ray == old {
                            return true;
                        }
                        if self.piece(ray).is_some() {
                            break;
                        }
                    }

                    ray = new;
                    while ray.west(1).is_some() {
                        ray = ray.west(1).unwrap();
                        if ray == old {
                            return true;
                        }
                        if self.piece(ray).is_some() {
                            break;
                        }
                    }
                }

                PieceType::Bishop => {
                    let mut ray = new;
                    while ray.northeast(1).is_some() {
                        ray = ray.northeast(1).unwrap();
                        if ray == old {
                            return true;
                        }
                        if self.piece(ray).is_some() {
                            break;
                        }
                    }

                    ray = new;
                    while ray.northwest(1).is_some() {
                        ray = ray.northwest(1).unwrap();
                        if ray == old {
                            return true;
                        }
                        if self.piece(ray).is_some() {
                            break;
                        }
                    }

                    ray = new;
                    while ray.southeast(1).is_some() {
                        ray = ray.southeast(1).unwrap();
                        if ray == old {
                            return true;
                        }
                        if self.piece(ray).is_some() {
                            break;
                        }
                    }

                    ray = new;
                    while ray.southwest(1).is_some() {
                        ray = ray.southwest(1).unwrap();
                        if ray == old {
                            return true;
                        }
                        if self.piece(ray).is_some() {
                            break;
                        }
                    }
                }

                PieceType::Queen => {
                    let mut ray = new;
                    while ray.north(1).is_some() {
                        ray = ray.north(1).unwrap();
                        if ray == old {
                            return true;
                        }
                        if self.piece(ray).is_some() {
                            break;
                        }
                    }

                    ray = new;
                    while ray.south(1).is_some() {
                        ray = ray.south(1).unwrap();
                        if ray == old {
                            return true;
                        }
                        if self.piece(ray).is_some() {
                            break;
                        }
                    }

                    ray = new;
                    while ray.east(1).is_some() {
                        ray = ray.east(1).unwrap();
                        if ray == old {
                            return true;
                        }
                        if self.piece(ray).is_some() {
                            break;
                        }
                    }

                    ray = new;
                    while ray.west(1).is_some() {
                        ray = ray.west(1).unwrap();
                        if ray == old {
                            return true;
                        }
                        if self.piece(ray).is_some() {
                            break;
                        }
                    }

                    ray = new;
                    while ray.northeast(1).is_some() {
                        ray = ray.northeast(1).unwrap();
                        if ray == old {
                            return true;
                        }
                        if self.piece(ray).is_some() {
                            break;
                        }
                    }

                    ray = new;
                    while ray.northwest(1).is_some() {
                        ray = ray.northwest(1).unwrap();
                        if ray == old {
                            return true;
                        }
                        if self.piece(ray).is_some() {
                            break;
                        }
                    }

                    ray = new;
                    while ray.southeast(1).is_some() {
                        ray = ray.southeast(1).unwrap();
                        if ray == old {
                            return true;
                        }
                        if self.piece(ray).is_some() {
                            break;
                        }
                    }

                    ray = new;
                    while ray.southwest(1).is_some() {
                        ray = ray.southwest(1).unwrap();
                        if ray == old {
                            return true;
                        }
                        if self.piece(ray).is_some() {
                            break;
                        }
                    }
                }
            }
        }
        false
    }

    pub fn nthreats(&self, pos: Pos, piece: Piece) -> isize {
        match piece.kind {
            PieceType::Pawn => self.pawn_nthreats(pos, piece.color),
            PieceType::Knight => self.knight_nthreats(pos, piece.color),
            PieceType::Bishop => self.bishop_nthreats(pos, piece.color),
            PieceType::Rook => self.rook_nthreats(pos, piece.color),
            PieceType::Queen => self.queen_nthreats(pos, piece.color),
            PieceType::King => self.king_nthreats(pos, piece.color),
        }
    }

    fn pawn_nthreats(&self, pos: Pos, color: Color) -> isize {
        let n = RefCell::new(0);
        let ref f = |ray| {
            match self.piece(ray) {
                Some(p) => {
                    if p.color == color.other() {
                        *n.borrow_mut() += 2
                    }
                }
                None => *n.borrow_mut() += 1,
            }
            if self.is_en_passant_target(ray) {
                *n.borrow_mut() += 2
            }
        };

        match color {
            Color::White => {
                pos.northeast(1).map(f);
                pos.northwest(1).map(f);
            }
            Color::Black => {
                pos.southeast(1).map(f);
                pos.southwest(1).map(f);
            }
        }

        // TODO: on an airplane, done know how to do this correctly
        let n = n.borrow_mut().clone();
        n
    }

    fn king_nthreats(&self, pos: Pos, color: Color) -> isize {
        let n = RefCell::new(0);

        let ref f = |ray| match self.piece(ray) {
            Some(p) => {
                if p.color == color.other() {
                    *n.borrow_mut() += 2
                }
            }
            None => *n.borrow_mut() += 1,
        };

        pos.mv(1, 1).map(f);
        pos.mv(1, 0).map(f);
        pos.mv(1, -1).map(f);
        pos.mv(0, 1).map(f);
        pos.mv(0, -1).map(f);
        pos.mv(-1, 1).map(f);
        pos.mv(-1, 0).map(f);
        pos.mv(-1, -1).map(f);

        // TODO: on an airplane, done know how to do this correctly
        let n = n.borrow_mut().clone();
        n
    }

    fn knight_nthreats(&self, pos: Pos, color: Color) -> isize {
        let n = RefCell::new(0);

        let ref f = |ray| match self.piece(ray) {
            Some(p) => {
                if p.color == color.other() {
                    *n.borrow_mut() += 2
                }
            }
            None => *n.borrow_mut() += 1,
        };

        pos.mv(1, 2).map(f);
        pos.mv(1, -2).map(f);
        pos.mv(-1, -2).map(f);
        pos.mv(-1, 2).map(f);
        pos.mv(2, 1).map(f);
        pos.mv(2, -1).map(f);
        pos.mv(-2, -1).map(f);
        pos.mv(-2, 1).map(f);

        // TODO: on an airplane, done know how to do this correctly
        let n = n.borrow_mut().clone();
        n
    }

    fn bishop_nthreats(&self, pos: Pos, color: Color) -> isize {
        let n = RefCell::new(0);

        let ref f = |ray| match self.piece(ray) {
            Some(p) => {
                if p.color == color.other() {
                    *n.borrow_mut() += 2
                }
            }
            None => *n.borrow_mut() += 1,
        };

        let mut ray = pos;
        while ray.northeast(1).is_some() {
            ray = ray.northeast(1).unwrap();
            f(ray);
        }

        ray = pos;
        while ray.northwest(1).is_some() {
            ray = ray.northwest(1).unwrap();
            f(ray);
        }

        ray = pos;
        while ray.southeast(1).is_some() {
            ray = ray.southeast(1).unwrap();
            f(ray);
        }

        ray = pos;
        while ray.southwest(1).is_some() {
            ray = ray.southwest(1).unwrap();
            f(ray);
        }

        // TODO: on an airplane, done know how to do this correctly
        let n = n.borrow_mut().clone();
        n
    }

    fn rook_nthreats(&self, pos: Pos, color: Color) -> isize {
        let n = RefCell::new(0);

        let ref f = |ray| match self.piece(ray) {
            Some(p) => {
                if p.color == color.other() {
                    *n.borrow_mut() += 2
                }
            }
            None => *n.borrow_mut() += 1,
        };

        let mut ray = pos;
        while ray.north(1).is_some() {
            ray = ray.north(1).unwrap();
            f(ray);
        }

        ray = pos;
        while ray.south(1).is_some() {
            ray = ray.south(1).unwrap();
            f(ray);
        }

        ray = pos;
        while ray.east(1).is_some() {
            ray = ray.east(1).unwrap();
            f(ray);
        }

        ray = pos;
        while ray.west(1).is_some() {
            ray = ray.west(1).unwrap();
            f(ray);
        }

        let n = n.borrow_mut().clone();
        n
    }

    fn queen_nthreats(&self, pos: Pos, color: Color) -> isize {
        let n = RefCell::new(0);

        let ref f = |ray| match self.piece(ray) {
            Some(p) => {
                if p.color == color.other() {
                    *n.borrow_mut() += 2
                }
            }
            None => *n.borrow_mut() += 1,
        };

        let mut ray = pos;
        while ray.north(1).is_some() {
            ray = ray.north(1).unwrap();
            f(ray);
        }

        ray = pos;
        while ray.south(1).is_some() {
            ray = ray.south(1).unwrap();
            f(ray);
        }

        ray = pos;
        while ray.east(1).is_some() {
            ray = ray.east(1).unwrap();
            f(ray);
        }

        ray = pos;
        while ray.west(1).is_some() {
            ray = ray.west(1).unwrap();
            f(ray);
        }

        ray = pos;
        while ray.northeast(1).is_some() {
            ray = ray.northeast(1).unwrap();
            f(ray);
        }

        ray = pos;
        while ray.northwest(1).is_some() {
            ray = ray.northwest(1).unwrap();
            f(ray);
        }

        ray = pos;
        while ray.southeast(1).is_some() {
            ray = ray.southeast(1).unwrap();
            f(ray);
        }

        ray = pos;
        while ray.southwest(1).is_some() {
            ray = ray.southwest(1).unwrap();
            f(ray);
        }

        let n = n.borrow_mut().clone();
        n
    }
}
