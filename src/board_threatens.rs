use board::Board;
use piece::{Color, PieceType};
use position::Pos;

impl Board {
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
}
