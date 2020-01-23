use combustion::board::Board;
use combustion::moves::Move;
use combustion::piece::Color;

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

    let mut to_move = Color::White;

    for mv_str in args.iter().skip(2) {
        // should be UCI format
        let mv = Move::from_xboard_format(mv_str, &board).unwrap();
        board = board.make_move(&mv).unwrap();
    }

    for mv in &board.legal_moves().unwrap() {
        println!("{}", mv.to_xboard_format(board.color_to_move));
    }
}
