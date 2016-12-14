// TODO:
// timer
// more fine grained score function
// hashing
// pondering

extern crate getopts;
extern crate libc;
extern crate num_cpus;
extern crate rand;
extern crate regex;

#[macro_use]
pub mod macros;

pub mod clock;
pub mod moves;
pub mod piece;
pub mod position;
pub mod threadpool;
pub mod util;

pub mod board;
pub mod board_alpha_beta;
pub mod board_from_fen;
pub mod board_moves;
pub mod board_tests;
pub mod board_threatens;

use board::Board;
use clock::Clock;
use moves::Move;
use piece::Color;
use util::ChessError;
use threadpool::Threadpool;

use std::env;
use std::io::Write;
use std::process::exit;
use std::sync::{Arc, Condvar, Mutex};
use std::sync::mpsc::{Sender, channel, TryRecvError};
use std::thread;
use std::time::Duration;
use std::rc::Rc;
use std::cell::RefCell;

use getopts::Options;
use libc::{signal, SIGINT, SIG_IGN};
use regex::Regex;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [OPTIONS]", program);
    print!("{}", opts.usage(&brief));
    exit(0);
}

fn ignore() {
    debug!("ignoring message");
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

    unsafe {
        signal(SIGINT, SIG_IGN); // ignore SIGINT!!!! xboard sends SIGINT WTF
    }

    let engine_random_choice = opts.opt_present("r");

    // main loop- recieving and sending messages to xboard
    debug!("combustion started! random={}", engine_random_choice);

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
    let re_usermove = Regex::new(r"^usermove ([\w\d]+)$").unwrap();

    let mut b = Board::initial();
    let mut force_mode = true;
    let mut max_depth = 0;
    let mut my_color = Color::Black;
    let mut history: Vec<Move> = Vec::new();

    let main_signal = Arc::new(Condvar::new());
    let main_mutex = Mutex::new(());
    let mut pool = Threadpool::new(num_cpus::get(), main_signal.clone());

    let white_clock = Rc::new(RefCell::new(Clock::new(30000, main_signal.clone()))); // init 5m
    let black_clock = Rc::new(RefCell::new(Clock::new(30000, main_signal.clone())));

    let mut my_clock    = white_clock.clone();
    let mut their_clock = black_clock.clone();

    let search_depth = 6;

    // let input_strings = Arc::new(Mutex::new(Vec::new()));
    let (tx, rx) = channel();
    let input_watcher_thread = stdin_watcher(tx, main_signal.clone());

    loop {
        debug!("TOP! clocks: mine={} theirs={}", *my_clock.borrow(), *their_clock.borrow());

        match rx.try_recv() {
            Err(TryRecvError::Disconnected) => panic!("channel disconnected!"),

            // only make a move if there are no commands to process
            Err(TryRecvError::Empty) => {
                if !force_mode && their_clock.borrow().is_zero() {
                    match my_color {
                        Color::White => send!("1-0 {{Flagged for time}}"),
                        Color::Black => send!("0-1 {{Flagged for time}}"),
                    }
                }

                if (engine_random_choice || pool.has_result()) && !force_mode && b.color_to_move == my_color {
                    debug!("getting result");

                    let mv_result;
                    if engine_random_choice {
                        mv_result = b.random_move();
                        thread::sleep(Duration::from_millis(500));
                    } else {
                        mv_result = pool.take_result().unwrap();
                    }

                    match mv_result {
                        Ok((mv,score)) => {
                            b = b.make_move(&mv).unwrap();
                            history.push(mv);
                            debug!("moving {} with score {}", mv, score);
                            debug!("new board:\n{}", b);
                            send!("move {}", mv.to_xboard_format(b.color_to_move));
                            my_clock.borrow().stop();
                            their_clock.borrow().start();
                        }
                        Err(ChessError::Stalemate) => {
                            send!("1/2-1/2 {{Stalemate}}");
                            force_mode = true;
                        }
                        Err(ChessError::Checkmate) => {
                            match my_color {
                                Color::White => send!("0-1 {{Checkmate}}"),
                                Color::Black => send!("1-0 {{Checkmate}}"),
                            }
                            force_mode = true;
                        }
                        Err(e) => send!("Error ({})", e),
                    }
                }

                // find a move if it is my turn
                else if !engine_random_choice && !pool.thinking() && !force_mode && b.color_to_move == my_color {
                    debug!("finding best move");
                    pool.find_best_move(&b, search_depth);
                }

                else {
                    // no input, no moves => wait
                    // debug!("sleep...");
                    let _ = main_signal.wait(main_mutex.lock().unwrap()).unwrap();
                    // let _ = main_signal.wait_timeout(main_mutex.lock().unwrap(), Duration::from_millis(500)).unwrap();
                }
            }

            Ok(s) => {
                debug!("recieved message: \"{}\"", s);

                if re_protover.is_match(&s) {
                    send!("feature usermove=1 sigint=0 ping=1 colors=0 playother=1 setboard=1 analyze=0 done=1");
                }

                else if re_ping.is_match(&s) {
                    let n = re_ping.captures(&s).unwrap()[1].parse::<usize>().unwrap();
                    // check that all previous commands are finished
                    while pool.thinking() {
                        thread::sleep(Duration::from_millis(50));
                    }
                    send!("pong {}", n);
                }

                else if s == "new" {
                    pool.abort_and_clear();
                    force_mode = false;
                    b = Board::initial();
                    max_depth = 0;
                    my_color = Color::Black;
                    // my clock is Black's
                    my_clock = black_clock.clone();
                    // other clock is White's
                    their_clock = white_clock.clone();
                    // reset clocks (stops clocks)
                    black_clock.borrow().reset();
                    white_clock.borrow().reset();
                    // use wall clock for time measurement.
                    // do not ponder now.
                    debug!("created new board:\n{}", b);
                }

                else if s == "force" { // accept moves from both sides, stop calculating
                    pool.abort_and_clear();
                    black_clock.borrow().stop();
                    white_clock.borrow().stop();
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
                    match my_color {
                        Color::White => {
                            my_clock    = white_clock.clone();
                            their_clock = black_clock.clone();
                        }
                        Color::Black => {
                            my_clock    = black_clock.clone();
                            their_clock = white_clock.clone();
                        }
                    }
                    // start engine's clock
                    my_clock.borrow().start();
                    // start thinking and make a move
                }

                else if s == "playother" {
                    // leave force mode
                    force_mode = false;
                    // play the color that is not on the move
                    my_color = b.color_to_move.other();
                    // opponent's clock is for the color on move
                    // my clock is clock for color not on move
                    match my_color {
                        Color::White => {
                            my_clock    = white_clock.clone();
                            their_clock = black_clock.clone();
                        }
                        Color::Black => {
                            my_clock    = black_clock.clone();
                            their_clock = white_clock.clone();
                        }
                    }
                    // start opponents clock
                    their_clock.borrow().start();
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
                    let csecs = re_time.captures(&s).unwrap()[1].parse::<isize>().unwrap();
                    my_clock.borrow_mut().correct(csecs);
                }

                else if re_otim.is_match(&s) { // set opponent clock time in centiseconds
                    let csecs = re_otim.captures(&s).unwrap()[1].parse::<isize>().unwrap();
                    their_clock.borrow_mut().correct(csecs);
                }

                else if s == "?" { // move now
                    // move now with best result
                    pool.abort();
                }

                else if s == "draw" {
                    // to accept: send "offer draw"
                    ignore();
                }

                // ^result ([012/]+-[012/]+|\*) (\{.*\})$
                else if re_result.is_match(&s) {
                    pool.abort_and_clear();
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

                else if re_usermove.is_match(&s) {
                    let ref mv_str = re_usermove.captures(&s).unwrap()[1];
                    match Move::from_xboard_format(mv_str, &b) {
                        Ok(mv) => {
                            debug!("got move {}", mv);
                            match b.make_move(&mv) {
                                Err(e) => {
                                    send!("Illegal move: ({}) {}", e, s);
                                }
                                Ok(new_board)  => {
                                    if !force_mode {
                                        // stop opponent's clock
                                        their_clock.borrow().stop();
                                        // start my clock
                                        my_clock.borrow().start();
                                    }
                                    // debug!("current board:\n{}", new_board);
                                    history.push(mv);
                                    b = new_board;
                                    // debug!("new board\n{}", b);
                                }
                            }
                        }
                        Err(e) => {
                            debug!("{}", e);
                            send!("Error (unknown command): {}", s);
                        }
                    }
                }

                else {
                    ignore();
                }
            }
        }
    }
}

fn next_input_line() -> String {
    let mut input = String::new();
    match std::io::stdin().read_line(&mut input) {
        Err(e) => panic!("[next_input_line]: {}", e),
        _ => {}
    }
    input.trim().to_string()
}

// fn stdin_watcher(strings: Arc<Mutex<Vec<String>>>, main_signal: Arc<Condvar>)
fn stdin_watcher(tx: Sender<String>, main_signal: Arc<Condvar>)
    -> thread::JoinHandle<()>
{
    thread::spawn(move || {
        loop {
            let s = next_input_line();
            tx.send(s).unwrap();
            main_signal.notify_all();
        }
    })
}
