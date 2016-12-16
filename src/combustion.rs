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
pub mod transposition_table;
pub mod util;

pub mod board;
pub mod board_alpha_beta;
pub mod board_from_fen;
pub mod board_moves;
pub mod board_tests;
pub mod board_threatens;
