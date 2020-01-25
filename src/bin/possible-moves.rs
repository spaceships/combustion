use combustion::board::Board;
use combustion::moves::Move;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut board = Board::initial();

    for mv_str in args.iter().skip(1) {
        // should be UCI format
        let mv = Move::from_xboard_format(mv_str, &board).unwrap();
        board = board.make_move(&mv).unwrap();
    }

    for mv in &board.legal_moves().unwrap() {
        println!("{}", mv.to_xboard_format(board.color_to_move));
    }
}
