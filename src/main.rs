extern crate rand;
extern crate regex;

#[macro_use]
pub mod util;
pub mod board;

use board::Board;
use std::io::{BufRead, Write};
use regex::Regex;

fn ignore() {
    debug!("ignoring message");
}

fn main() {
    // main loop- recieving and sending messages to xboard
    debug!("combustion started!");
    let stdin = ::std::io::stdin();

    // compile regexes
    let re_new   = Regex::new(r"^new$").unwrap();
    let re_quit  = Regex::new(r"^quit$").unwrap();
    let re_force = Regex::new(r"^force$").unwrap();
    let re_go    = Regex::new(r"^go$").unwrap();
    let re_level = Regex::new(r"^level (\d+) (\d+)(:\d+)? (\d+)$").unwrap();
    let re_st    = Regex::new(r"^st (\d+)$").unwrap();
    let re_sd    = Regex::new(r"^sd (\d+)$").unwrap();
    let re_time  = Regex::new(r"^time (\d+)$").unwrap();
    let re_otim  = Regex::new(r"^otim (\d+)$").unwrap();

    let re_accepted = Regex::new(r"^accepted$").unwrap();
    let re_rejected = Regex::new(r"^rejected$").unwrap();

    let mut b = Board::initial();

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let s = line.trim();
        debug!("recieved message: \"{}\"", s);

        if re_quit.is_match(s) { break }
        else if re_force.is_match(s) { ignore() } // accept moves from both sides, stop calculating
        else if re_go.is_match(s) { ignore() } // return from force mode

        else if re_new.is_match(s) {
            b = Board::initial();
            debug!("created new board:\n{}", b);
        }

        else if re_level.is_match(s) { // setting the clock mode
            ignore();
        }

        else if re_st.is_match(s) { // set the delay
            ignore();
        }

        else if re_sd.is_match(s) { // set the max-depth
            ignore();
        }

        else if re_time.is_match(s) { // set my clock time
            ignore();
        }

        else if re_otim.is_match(s) { // set their clock time
            ignore();
        }

        else { ignore() }

    }

}
