use std::fmt;

#[derive(Debug)]
pub enum ChessError {
    ParseError(String),
    IllegalMove(String),
    BadBoardState(String),
    Stalemate,
    Checkmate,
}

impl ChessError {
    pub fn msg(&self) -> String {
        match *self {
            ChessError::ParseError(ref s) => s.clone(),
            ChessError::IllegalMove(ref s) => s.clone(),
            ChessError::BadBoardState(ref s) => s.clone(),
            ChessError::Stalemate => "stalemate".to_string(),
            ChessError::Checkmate => "checkmate".to_string(),
        }
    }
}

impl fmt::Display for ChessError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg())
    }
}

pub fn to_algebra(coord: usize) -> Result<String, ChessError> {
    if coord >= 64 {
        parse_error!("[to_algebra] coordinate out of bounds! got \"{}\"", coord);
    }
    let x = coord as u8 % 8;
    let y = (coord as u8 - x) / 8;
    let rank = ('1' as u8 + (7 - y)) as char;
    let file = ('a' as u8 + x) as char;
    let mut s = String::with_capacity(2);
    s.push(file);
    s.push(rank);
    Ok(s)
}

pub fn from_algebra(s: &str) -> Result<usize, ChessError> {
    let cs: Vec<char> = s.chars().collect();
    if (cs[1] as usize) < '1' as usize || cs[1] as usize > '8' as usize ||
       (cs[0] as usize) < 'a' as usize || cs[0] as usize > 'h' as usize {
        parse_error!("[from_algebra] parse error: \"{}\"", s);
    }
    let row = 7 - (cs[1] as usize - '1' as usize);
    let col = cs[0] as usize - 'a' as usize;
    Ok(row * 8 + col)
}

#[cfg(test)]
mod tests {
    use util::{to_algebra, from_algebra};
    use rand::{self, Rng};

    #[test]
    fn coordinates_to_algebra() {
        assert_eq!(from_algebra("e4").unwrap(), 4*8 + 4);
        assert_eq!(from_algebra("h8").unwrap(), 0*8 + 7);
        assert_eq!(from_algebra("a8").unwrap(), 0*8 + 0);
        assert_eq!(from_algebra("a1").unwrap(), 7*8 + 0);
        assert_eq!(from_algebra("f3").unwrap(), 5*8 + 5);
        assert_eq!(from_algebra("c2").unwrap(), 6*8 + 2);
        assert_eq!(from_algebra("c7").unwrap(), 1*8 + 2);
        let mut rng = rand::thread_rng();
        for _ in 0..16 {
            let x = rng.gen::<usize>() % 64;
            assert_eq!(x, from_algebra(&to_algebra(x).unwrap()).unwrap());
        }
    }
}
