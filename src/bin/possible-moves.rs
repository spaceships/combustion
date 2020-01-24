use combustion::board::Board;
use combustion::moves::Move;

use itertools::Itertools;
use std::env;
use std::process::exit;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Error: at least one argument required. Exiting...");
        exit(1);
    }

    let mut board = if args[1] == "startpos" {
        Board::initial()
    } else {
        Board::from_fen(&args[1]).unwrap()
    };

    for mv_str in args.iter().skip(2) {
        // should be UCI format
        let mv = Move::from_xboard_format(mv_str, &board).unwrap();
        board = board.make_move(&mv).unwrap();
    }

    println!(
        "{}",
        board
            .legal_moves()
            .unwrap()
            .iter()
            .map(|mv| mv.to_xboard_format(board.color_to_move))
            .join(" ")
    );
}
