use std::io;
use std::io::{BufRead, Write};

macro_rules! debug(
    ($($arg:tt)*) => { {
        let r = writeln!(&mut io::stderr(), $($arg)*);
        r.expect("failed printing to stderr");
    } }
);

fn main() {

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let s = line.trim();
        debug!("recieved message: \"{}\"", s);
        println!("whats up!!!!!!");
    }

}
