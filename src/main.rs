use std::io;
use std::io::{BufRead, Write};

pub mod board;
use board::Board;

macro_rules! debug(
    ($($arg:tt)*) => { {
        let mut stderr = io::stderr();
        let r = writeln!(&mut stderr, $($arg)*);
        r.expect("failed printing to stderr");
        let r = stderr.flush();
        r.expect("failed flushing stderr");
    } }
);

macro_rules! send(
    ($($arg:tt)*) => { {
        let mut stdout = io::stdout();
        let mut stderr = io::stderr();
        let s = format!($($arg)*);
        stdout.write(s.as_str().as_bytes()).expect("failed printing to stdout");
        stdout.write("\n".as_bytes()).expect("failed printing to stdout");
        let debug = "sent message: \"".to_string() + &s + "\"\n";
        stderr.write(debug.as_str().as_bytes()).expect("failed printing to stderr");
        stderr.flush().expect("failed flushing stderr");
        stdout.flush().expect("failed flushing stdout");
    } }
);

fn main() {
    // main loop- recieving and sending messages to xboard
    debug!("combustion started!");
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let s = line.trim();
        debug!("recieved message: \"{}\"", s);
        println!("whats up!!!!!!");
        match s {
            "xboard" => send!(""),
            "new" => {
                let b = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
                debug!("{}", b);
            }
            _ => debug!("ignoring message"),
        }
    }

}
