extern crate rand;

#[macro_use]
pub mod util;
pub mod board;

use board::Board;
use std::io::{BufRead, Write};

fn main() {
    // main loop- recieving and sending messages to xboard
    debug!("combustion started!");
    let stdin = ::std::io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let s = line.trim();
        debug!("recieved message: \"{}\"", s);
        println!("whats up!!!!!!");
        match s {
            "xboard" => send!(""),
            "new" => {
                let b = Board::initial();
                debug!("{}", b);
            }
            _ => debug!("ignoring message"),
        }
    }

}
