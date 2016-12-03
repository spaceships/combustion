extern crate rand;
extern crate regex;
extern crate libc;

#[macro_use]
pub mod util;
pub mod board;

use board::{Board, Move};
use std::io::{BufRead, Write};
use regex::Regex;
use rand::Rng;

use libc::{signal, SIGINT, SIG_IGN};

fn ignore() {
    debug!("ignoring message");
}

fn main() {
    // main loop- recieving and sending messages to xboard
    debug!("combustion started!");

    unsafe {
        signal(SIGINT, SIG_IGN); // ignore SIGINT!!!! xboard sends SIGINT
    }

    let mut rng = rand::thread_rng();

    // compile regexes
    let re_level = Regex::new(r"^level (\d+) (\d+)(:\d+)? (\d+)$").unwrap();
    let re_st    = Regex::new(r"^st (\d+)$").unwrap();
    let re_sd    = Regex::new(r"^sd (\d+)$").unwrap();
    let re_time  = Regex::new(r"^time (\d+)$").unwrap();
    let re_otim  = Regex::new(r"^otim (\d+)$").unwrap();
    let re_protover  = Regex::new(r"^protover (\d+)$").unwrap();
    let re_variant  = Regex::new(r"^variant (\w+)$").unwrap();
    let re_ping  = Regex::new(r"^ping (\d+)$").unwrap();
    let re_result  = Regex::new(r"^result ([012/]+-[012/]+) (\{.*\})$").unwrap();
    let re_setboard = Regex::new(r"^setboard (.*)$").unwrap();


    let mut b = Board::initial();

    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let s = line.trim();
        debug!("recieved message: \"{}\"", s);

        if s == "xboard" { ignore() }
        else if s == "accepted" { ignore() }
        else if s == "rejected" { ignore() }
        else if s == "random" { ignore() }
        else if s == "white" { ignore() }
        else if s == "black" { ignore() }
        else if re_variant.is_match(s) { ignore() }
        else if re_protover.is_match(s) { ignore() }

        else if re_ping.is_match(s) { ignore() }
        else if re_result.is_match(s) { ignore() }
        else if re_setboard.is_match(s) { ignore() }

        else if s == "hard" { ignore() } // turn on pondering
        else if s == "easy" { ignore() } // turn on pondering
        else if s == "post" { ignore() } // turn on thinking/pondering output
        else if s == "nopost" { ignore() } // turn off thinking/pondering ouptut

        else if s == "draw" { ignore() }

        else if s == "force" { ignore() } // accept moves from both sides, stop calculating
        else if s == "go" { ignore() } // return from force mode
        else if s == "quit" { ignore() } // return from force mode
        else if s == "?" { ignore() } // move now

        else if s == "new" {
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

        else {
            match Move::from_xboard_format(s, &b) {
                Ok(m) => {
                    debug!("got move {}", m);
                    match b.make_move(&m) {
                        Err(e) => {
                            send!("Illegal move: ({}) {}", e.msg(), s);
                        }
                        Ok(new_board)  => {
                            debug!("current:board:\n{}", new_board);
                            let ms = new_board.legal_moves();
                            if ms.len() > 0 {
                                let i = rng.gen::<usize>() % ms.len();
                                send!("move {}", ms[i].to_xboard_format(new_board.color_to_move));
                                b = new_board.make_move(&ms[i]).unwrap();
                                debug!("current board:\n{}", b);
                            } else {
                                send!("resign");
                            }
                        }
                    }
                }
                Err(e) => {
                    debug!("{}", e.msg());
                    debug!("not a move: {}", s);
                    ignore();
                }
            }
        }
    }
}
