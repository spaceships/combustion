use combustion::board::Board;

use std::env;
use std::process::exit;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Error: exactly one argument required. Exiting...");
        exit(1);
    }


    let board = Board::from_fen(&args[1]).unwrap();

    for mv in &board.legal_moves().unwrap() {
        println!("{}", mv);
    }
}
