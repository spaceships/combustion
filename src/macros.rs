#[macro_export]
macro_rules! mv {
    ( $str: expr ) => {{
        Move::from_algebra($str).unwrap()
    }};
}

#[macro_export]
macro_rules! pos {
    ( $str: expr ) => {{
        Pos::from_algebra($str).unwrap()
    }};
}

#[macro_export]
macro_rules! parse_error(
    ($($arg:tt)*) => { {
        let s = format!($($arg)*);
        return Err(ChessError::ParseError(s));
    } }
);

#[macro_export]
macro_rules! illegal_move_error(
    ($($arg:tt)*) => { {
        let s = format!($($arg)*);
        return Err(ChessError::IllegalMove(s));
    } }
);

#[macro_export]
macro_rules! board_state_error(
    ($($arg:tt)*) => { {
        let s = format!($($arg)*);
        return Err(ChessError::BadBoardState(s));
    } }
);

#[macro_export]
macro_rules! debug(
    ($($arg:tt)*) => { {
        let mut stderr = ::std::io::stderr();
        let r = write!(&mut stderr, "# ");
        r.expect("failed printing to stderr");
        let r = writeln!(&mut stderr, $($arg)*);
        r.expect("failed printing to stderr");
        let r = stderr.flush();
        r.expect("failed flushing stderr");
    } }
);

#[macro_export]
macro_rules! send(
    ($($arg:tt)*) => { {
        let mut stdout = ::std::io::stdout();
        // let mut stderr = ::std::io::stderr();
        let s = format!($($arg)*);
        stdout.write(s.as_str().as_bytes()).expect("failed printing to stdout");
        stdout.write("\n".as_bytes()).expect("failed printing to stdout");
        // let debug = "sent message: \"".to_string() + &s + "\"\n";
        // stderr.write(debug.as_str().as_bytes()).expect("failed printing to stderr");
        // stderr.flush().expect("failed flushing stderr");
        stdout.flush().expect("failed flushing stdout");
    } }
);
