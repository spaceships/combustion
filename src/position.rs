use util::{ChessError, to_algebra, from_algebra};

use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Pos(usize);

impl Pos {
    pub fn new(rank: usize, file: usize) -> Pos {
        Pos(rank * 8 + file)
    }

    pub fn zero() -> Pos {
        Pos(0)
    }

    pub fn from_index(ix: usize) -> Pos {
        Pos(ix)
    }

    pub fn index(&self) -> usize {
        self.0
    }

    pub fn from_algebra(s: &str) -> Result<Self, ChessError> {
        Ok(Pos(from_algebra(s)?))
    }

    pub fn to_algebra(&self) -> String {
        to_algebra(self.0).unwrap()
    }

    pub fn file(&self) -> usize {
        self.0 % 8
    }

    pub fn rank(&self) -> usize {
        (self.0 - self.file()) / 8
    }

    pub fn file_is(&self, c: char) -> bool {
        let file = c as usize - 'a' as usize;
        self.file() == file
    }

    pub fn rank_is(&self, c: usize) -> bool {
        let rank = 8 - c;
        self.rank() == rank
    }

    // north is negative, west is negative
    pub fn mv(&self, vertical: isize, horizontal: isize) -> Option<Pos> {
        let rank = self.rank() as isize + vertical;
        let file = self.file() as isize + horizontal;
        if rank >= 0 && rank < 8 && file >= 0 && file < 8 {
            Some(Pos::new(rank as usize, file as usize))
        } else {
            None
        }
    }

    pub fn north(&self, d: isize) -> Option<Pos> { self.mv(-d, 0) }
    pub fn south(&self, d: isize) -> Option<Pos> { self.mv( d, 0) }
    pub fn east(&self, d: isize)  -> Option<Pos> { self.mv( 0, d) }
    pub fn west(&self, d: isize)  -> Option<Pos> { self.mv( 0,-d) }
    pub fn northeast(&self, d: isize) -> Option<Pos> { self.mv(-d, d) }
    pub fn northwest(&self, d: isize) -> Option<Pos> { self.mv(-d,-d) }
    pub fn southeast(&self, d: isize) -> Option<Pos> { self.mv( d, d) }
    pub fn southwest(&self, d: isize) -> Option<Pos> { self.mv( d,-d) }

    pub fn value(&self) -> isize {
        if self.rank_is(1) || self.file_is('a') || self.rank_is(8) || self.file_is('h') {
            return 0
        }
        if self.rank_is(2) || self.file_is('b') || self.rank_is(7) || self.file_is('g'){
            return 1
        }
        if self.rank_is(3) || self.file_is('c') || self.rank_is(6) || self.file_is('f'){
            return 2
        }
        else {
            return 3
        }
    }
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", to_algebra(self.0).unwrap())
    }
}
