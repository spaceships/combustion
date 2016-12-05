extern crate rand;
extern crate regex;
extern crate getopts;

#[macro_use]
pub mod util;
pub mod board;

use board::{Board, Move, Color};
use util::ChessError;

use getopts::Options;
use regex::Regex;
use std::env;
use std::io::Write;
use std::process::exit;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [OPTIONS]", program);
    print!("{}", opts.usage(&brief));
    exit(0);
}

fn ignore() {
    debug!("ignoring message");
}

fn next_input_line() -> String {
    let mut input = String::new();
    match std::io::stdin().read_line(&mut input) {
        Err(e) => panic!("[next_input_line]: {}", e),
        _ => {}
    }
    input.trim().to_string()
}

#[allow(unused_variables, unused_assignments)]
fn main() {
    let args: Vec<String> = env::args().collect();
    let mut options = Options::new();
    options.optflag("h", "help", "Print this help menu.");
    options.optflag("r", "random", "Choose moves randomly.");
    let opts = options.parse(&args[1..]).unwrap();
    if opts.opt_present("h") {
        print_usage(&args[0], options);
    }

    let engine_random_choice = opts.opt_present("r");

    // main loop- recieving and sending messages to xboard
    debug!("combustion started!");

    // precompile regexes
    let re_level    = Regex::new(r"^level (\d+) (\d+)(:\d+)? (\d+)$").unwrap();
    let re_st       = Regex::new(r"^st (\d+)$").unwrap();
    let re_sd       = Regex::new(r"^sd (\d+)$").unwrap();
    let re_time     = Regex::new(r"^time (\d+)$").unwrap();
    let re_otim     = Regex::new(r"^otim (\d+)$").unwrap();
    let re_protover = Regex::new(r"^protover (\d+)$").unwrap();
    let re_variant  = Regex::new(r"^variant (\w+)$").unwrap();
    let re_ping     = Regex::new(r"^ping (\d+)$").unwrap();
    let re_result   = Regex::new(r"^result ([012/]+-[012/]+) (\{.*\})$").unwrap();
    let re_setboard = Regex::new(r"^setboard (.+)$").unwrap();
    let re_accepted = Regex::new(r"^accepted (\w+)$").unwrap();
    let re_rejected = Regex::new(r"^rejected (\w+)$").unwrap();
    let re_nps      = Regex::new(r"^nps (\d+)$").unwrap();
    let re_name     = Regex::new(r"^name (.+)$").unwrap();
    let re_rating   = Regex::new(r"^rating (\d+) (\d+)$").unwrap();

    let mut b = Board::initial();
    let mut force_mode = true;
    let mut max_depth = 0;
    let mut my_color = Color::Black;
    let mut history: Vec<Move> = Vec::new();

    loop {
        let s = next_input_line();
        debug!("recieved message: \"{}\"", s);

        if s == "xboard" || s == "random" || s == "white" || s == "black" ||
            s == "quit" || s == "hint" || s == "bk" || s == "computer" ||
            re_variant.is_match(&s) ||
            re_accepted.is_match(&s) ||
            re_rejected.is_match(&s) ||
            re_nps.is_match(&s) ||
            re_name.is_match(&s) ||
            re_rating.is_match(&s)
        {
            ignore();
        }

        else if re_protover.is_match(&s) {
            send!("feature sigint=0 ping=1 playother=1 setboard=1 analyze=0 done=1");
        }

        else if re_ping.is_match(&s) {
            let n = re_ping.captures(&s).unwrap()[1].parse::<usize>().unwrap();
            // TODO: check that all previous commands are finished
            send!("pong {}", n);
        }

        else if s == "new" {
            force_mode = false;
            b = Board::initial();
            max_depth = 0;
            my_color = Color::Black;
            // my clock is Black's
            // other clock is White's
            // reset clocks.
            // use wall clock for time measurement.
            // stop clocks.
            // do not ponder now.
            debug!("created new board:\n{}", b);
        }

        else if s == "force" { // accept moves from both sides, stop calculating
            // stop clocks
            // still: check moves are legal and made in proper turn
            force_mode = true;
        }

        else if s == "go" {
            // leave force mode
            force_mode = false;
            // play as the color that is on move
            my_color = b.color_to_move;
            // that color's clock is mine
            // opponent's clock is the other color
            // start engine's clock
            // start thinking and make a move
        }

        else if s == "playother" {
            // leave force mode
            force_mode = false;
            // play the color that is not on the move
            my_color = b.color_to_move.other();
            // opponent's clock is for the color on move
            // my clock is clock for color not on move
            // start opponents clock
            // begin pondering
            // wait for opponent's move
        }

        // ^level (\d+) (\d+)(:\d+)? (\d+)$
        else if re_level.is_match(&s) { // setting the clock mode
            let caps = re_level.captures(&s).unwrap();
            ignore();
        }

        else if re_st.is_match(&s) { // set the delay
            ignore();
        }

        else if re_sd.is_match(&s) { // set the max-depth
            max_depth = re_sd.captures(&s).unwrap()[1].parse::<usize>().unwrap();
            debug!("set max search depth to {}", max_depth);
        }

        // clocks always remain with color
        // which one to update is determined by which side i play
        else if re_time.is_match(&s) { // set my clock time in centiseconds
            // how many 1/100ths of a second do i have
            let centiseconds = re_time.captures(&s).unwrap()[1].parse::<usize>().unwrap();
            ignore();
        }

        else if re_otim.is_match(&s) { // set opponent clock time in centiseconds
            let centiseconds = re_otim.captures(&s).unwrap()[1].parse::<usize>().unwrap();
            ignore();
        }

        else if s == "?" { // move now
            // move now with best result
            // set global move_now flag
            // worker threads check it and give their best results
            ignore()
        }

        else if s == "draw" {
            // to accept: send "offer draw"
            ignore();
        }

        // ^result ([012/]+-[012/]+|\*) (\{.*\})$
        else if re_result.is_match(&s) {
            // TODO: implement this-- allows aborting
            ignore()
        }

        // ^setboard (.+)$
        else if re_setboard.is_match(&s) {
            let ref fen = re_setboard.captures(&s).unwrap()[1];
            match Board::from_fen(fen) {
                Ok(new_board) => {
                    debug!("set board to new position\n{}", new_board);
                    b = new_board;
                }
                Err(e) => {
                    debug!("{}", e.msg());
                }
            }
        }

        else if s == "undo" {
            if history.len() > 0 {
                history.pop();
            }
            b = Board::initial();
            for mv in history.iter() {
                b = b.make_move(mv).unwrap();
            }
        }

        else if s == "remove" {
            if history.len() > 0 {
                history.pop();
            }
            if history.len() > 0 {
                history.pop();
            }
            b = Board::initial();
            for mv in history.iter() {
                b = b.make_move(mv).unwrap();
            }
        }

        else if s == "hard" { ignore() } // turn on pondering
        else if s == "easy" { ignore() } // turn off pondering
        else if s == "post" { ignore() } // turn on thinking/pondering output
        else if s == "nopost" { ignore() } // turn off thinking/pondering ouptut

        else {
            match Move::from_xboard_format(&s, &b) {
                Ok(mv) => {
                    debug!("got move {}", mv);
                    match b.make_move(&mv) {
                        Err(e) => {
                            send!("Illegal move: ({}) {}", e, s);
                        }
                        Ok(new_board)  => {
                            if !force_mode {
                                // stop opponent's clock
                                // start my clock
                            }
                            // debug!("current board:\n{}", new_board);
                            history.push(mv);
                            b = new_board;
                        }
                    }
                }
                Err(e) => {
                    debug!("{}", e);
                    send!("Error (unknown command): {}", s);
                }
            }
        }

        if !force_mode && b.color_to_move == my_color {
            let mv_result;
            if engine_random_choice {
                mv_result = b.random_move();
            } else {
                mv_result = b.best_move(my_color, 4);
            }
            match mv_result {
                Ok((mv,score)) => {
                    send!("move {}", mv.to_xboard_format(b.color_to_move));
                    b = b.make_move(&mv).unwrap();
                    history.push(mv);
                    debug!("made move {} with score {}", mv, score);
                    debug!("current board:\n{}", b);
                }
                Err(ChessError::Stalemate) => {
                    send!("1/2-1/2 {{Stalemate}}");
                }
                Err(ChessError::Checkmate) => {
                    match my_color {
                        Color::White => send!("0-1 {{Checkmate}}"),
                        Color::Black => send!("1-0 {{Checkmate}}"),
                    }
                }
                Err(e) => send!("Error ({}): {}", e, s),
            }
        }
    }
}
