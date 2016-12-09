#[cfg(test)]
mod tests {
    use board::Board;
    use position::Pos;
    use moves::Move;
    use piece::Color;

    use std::collections::HashSet;

    macro_rules! legal_moves_are {
        ( $fen:expr, $($mv:expr),* ) => {{
            let b = Board::from_fen($fen).expect("[legal_moves] bad fen!");
            println!("\n{}", b);
            let mut should_be = HashSet::new();
            $(
                should_be.insert(Move::from_algebra($mv).expect("[legal_moves] bad move!"));
            );*
            for mv in b.legal_moves().unwrap() {
                println!("{}", mv);
            }
            let res: HashSet<Move> = b.legal_moves().unwrap().into_iter().collect();
            assert_eq!(should_be, res);
            b
        }};
    }

    macro_rules! legal_moves_include {
        ( $fen:expr, $($mv:expr),* ) => {{
            let b = Board::from_fen($fen).expect("[legal_moves] bad fen!");
            println!("\n{}", b);
            let mut should_be = HashSet::new();
            $(
                should_be.insert(Move::from_algebra($mv).expect("[legal_moves] bad move!"));
            );*
            let res: HashSet<Move> = b.legal_moves().unwrap().into_iter().collect();
            assert!(should_be.is_subset(&res));
            b
        }};
    }

    macro_rules! board_after_move_is {
        ($mv:expr, $initial:expr, $result:expr) => {{
            let b = Board::from_fen($initial).expect("[board_after_move_is] bad fen!");
            assert_eq!(
                b.make_move(&Move::from_algebra($mv).expect("[board_after_move_is] bad move")).unwrap(),
                Board::from_fen($result).expect("[board_after_move_is] bad fen!")
            );
        }};
    }

    #[test]
    fn initial_moves() {
        let b = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        println!("\n{}", b);
        assert_eq!(b.legal_moves().unwrap().len(), 20);
    }

    #[test]
    fn white_pawn() {
        legal_moves_are!("8/8/8/8/3p4/p1p5/1P1P4/8 w - - 0 1",
            "b2-b3", "b2-b4", "b2xa3", "b2xc3", "d2xc3", "d2-d3");
        board_after_move_is!("b2-b3",
            "8/8/8/8/3p4/p1p5/1P1P4/8 w - - 0 1",
            "8/8/8/8/3p4/pPp5/3P4/8 b - - 0 1");
    }

    #[test]
    fn black_pawn() {
        legal_moves_are!("8/2p4p/1P1P4/7P/8/8/8/8 b - - 0 1",
            "c7-c5", "c7-c6", "c7xb6", "c7xd6", "h7-h6");
        board_after_move_is!("c7-c5",
            "8/2p4p/1P1P4/7P/8/8/8/8 b - - 0 1",
            "8/7p/1P1P4/2p4P/8/8/8/8 w - - 0 2");
    }

    #[test]
    fn white_en_passant() {
        let b = legal_moves_are!("8/8/8/pP6/8/8/8/8 w - a6 0 1",
            "b5-b6", "b5xa6e.p.");
        assert!(b.threatens(Color::White, pos!("a5")));
        board_after_move_is!("b5xa6e.p.",
            "8/8/8/pP6/8/8/8/8 w - a6 0 1",
            "8/8/P7/8/8/8/8/8 b - - 0 1");
    }

    #[test]
    fn black_en_passant() {
        let b = legal_moves_are!("8/8/8/8/pP6/8/8/8 b - b3 0 1", "a4-a3", "a4xb3e.p.");
        assert!(b.threatens(Color::Black, pos!("b3")));
        board_after_move_is!("a4xb3e.p.",
            "8/8/8/8/pP6/8/8/8 b - b3 0 1",
            "8/8/8/8/8/1p6/8/8 w - - 0 2");
    }

    #[test]
    fn white_promotion() {
        legal_moves_are!("3n4/4P3/8/8/8/8/8/8 w - - 0 1",
            "e7-e8=Q", "e7-e8=N", "e7-e8=R", "e7-e8=B", "e7xd8=Q", "e7xd8=N", "e7xd8=R", "e7xd8=B");
        board_after_move_is!("e7-e8=N",
            "3n4/4P3/8/8/8/8/8/8 w - - 0 1",
            "3nN4/8/8/8/8/8/8/8 b - - 0 1");
    }

    #[test]
    fn black_promotion() {
        legal_moves_are!("8/8/8/8/8/8/3p4/4N3/ b - - 0 1",
            "d2-d1=Q", "d2-d1=N", "d2-d1=R", "d2-d1=B", "d2xe1=Q", "d2xe1=N", "d2xe1=R", "d2xe1=B");
        board_after_move_is!("d2xe1=B",
            "8/8/8/8/8/8/3p4/4N3/ b - - 0 1",
            "8/8/8/8/8/8/8/4b3 w - - 0 2");
    }

    #[test]
    fn white_queen() {
        legal_moves_are!("3n1q3/4Q3/8/4p3/7P/p7/8/8 w - - 0 1",
            "Qe7xd8", "Qe7xf8", "Qe7xe5", "Qe7xa3", "Qe7-e6", "Qe7-e8",
            "Qe7-a7", "Qe7-b7", "Qe7-c7", "Qe7-d7", "Qe7-f7", "Qe7-g7",
            "Qe7-h7", "Qe7-f6", "Qe7-g5", "Qe7-d6", "Qe7-c5", "Qe7-b4",
            "h4-h5");
        board_after_move_is!("Qe7-b4",
            "3n1q3/4Q3/8/4p3/7P/p7/8/8 w - - 0 1",
            "3n1q3/8/8/4p3/1Q5P/p7/8/8 b - - 1 1");
    }

    #[test]
    fn black_queen() {
        legal_moves_are!("3N1Q3/4q3/8/4P3/7p/P7/8/8 b - - 0 1",
            "Qe7xd8", "Qe7xf8", "Qe7xe5", "Qe7xa3", "Qe7-e6", "Qe7-e8",
            "Qe7-a7", "Qe7-b7", "Qe7-c7", "Qe7-d7", "Qe7-f7", "Qe7-g7",
            "Qe7-h7", "Qe7-f6", "Qe7-g5", "Qe7-d6", "Qe7-c5", "Qe7-b4",
            "h4-h3");
        board_after_move_is!("Qe7-b4",
            "3N1Q3/4q3/8/4P3/7p/P7/8/8 b - - 0 1",
            "3N1Q3/8/8/4P3/1q5p/P7/8/8 w - - 1 2");
    }

    #[test]
    fn white_rook() {
        legal_moves_are!("3n1q3/4R3/8/4p3/7P/p7/8/8 w - - 0 1",
            "Re7xe5", "Re7-e6", "Re7-e8", "Re7-a7", "Re7-b7", "Re7-c7",
            "Re7-d7", "Re7-f7", "Re7-g7", "Re7-h7", "h4-h5");
        board_after_move_is!("Re7-f7",
            "3n1q3/4R3/8/4p3/7P/p7/8/8 w - - 0 1",
            "3n1q3/5R2/8/4p3/7P/p7/8/8 b - - 1 1");
    }

    #[test]
    fn black_rook() {
        legal_moves_are!("3N1Q3/4r3/8/4P3/7p/P7/8/8 b - - 0 1",
            "Re7xe5", "Re7-e6", "Re7-e8", "Re7-a7", "Re7-b7", "Re7-c7",
            "Re7-d7", "Re7-f7", "Re7-g7", "Re7-h7", "h4-h3");
        board_after_move_is!("Re7-f7",
            "3N1Q3/4r3/8/4P3/7p/P7/8/8 b - - 0 1",
            "3N1Q3/5r2/8/4P3/7p/P7/8/8 w - - 1 2");
    }

    #[test]
    fn white_bishop() {
        legal_moves_are!("3n1q3/4B3/8/4p3/7P/p7/8/8 w - - 0 1",
            "Be7xd8", "Be7xf8", "Be7xa3", "Be7-f6", "Be7-g5", "Be7-d6",
            "Be7-c5", "Be7-b4", "h4-h5");
        board_after_move_is!("Be7xd8",
            "3n1q3/4B3/8/4p3/7P/p7/8/8 w - - 0 1",
            "3B1q3/8/8/4p3/7P/p7/8/8 b - - 0 1");
    }

    #[test]
    fn black_bishop() {
        legal_moves_are!("3N1Q3/4b3/8/4P3/7p/P7/8/8 b - - 0 1",
            "Be7xd8", "Be7xf8", "Be7xa3", "Be7-f6", "Be7-g5", "Be7-d6", "Be7-c5", "Be7-b4", "h4-h3");
        board_after_move_is!("Be7xd8",
            "3N1Q3/4b3/8/4P3/7p/P7/8/8 b - - 0 1",
            "3b1Q3/8/8/4P3/7p/P7/8/8 w - - 0 2");
    }

    #[test]
    fn white_knight() {
        legal_moves_are!("N7/2p5/8/8/4N3/2p5/8/8 w - - 0 1",
            "Na8xc7", "Na8-b6", "Ne4-c5", "Ne4xc3", "Ne4-d6", "Ne4-d2",
            "Ne4-f6", "Ne4-f2", "Ne4-g5", "Ne4-g3");
        board_after_move_is!("Ne4-c5",
            "N7/2p5/8/8/4N3/2p5/8/8 w - - 0 1",
            "N7/2p5/8/2N5/8/2p5/8/8 b - - 1 1");
    }

    #[test]
    fn black_knight() {
        legal_moves_are!("n7/2P5/8/8/4n3/2P5/8/8 b - - 0 1",
            "Na8xc7", "Na8-b6", "Ne4-c5", "Ne4xc3", "Ne4-d6", "Ne4-d2",
            "Ne4-f6", "Ne4-f2", "Ne4-g5", "Ne4-g3");
        board_after_move_is!("Ne4-c5",
            "n7/2P5/8/8/4n3/2P5/8/8 b - - 0 1",
            "n7/2P5/8/2n5/8/2P5/8/8 w - - 1 2");
    }

    #[test]
    fn white_king() {
        legal_moves_are!("n/1p6/2KP5/1p6/8/8/8/8 w - - 0 1",
            "Kc6xb7", "Kc6-d7", "Kc6-d5", "Kc6xb5", "Kc6-c5");
        board_after_move_is!("Kc6xb7",
            "n/1p6/2KP5/1p6/8/8/8/8 w - - 0 1",
            "n/1K6/3P5/1p6/8/8/8/8 b - - 0 1");
    }

    #[test]
    fn black_king() {
        legal_moves_are!("N/1P6/2kp5/1P6/8/8/8/8 b - - 0 1",
            "Kc6xb7", "Kc6-d7", "Kc6-d5", "Kc6xb5", "Kc6-c5");
        board_after_move_is!("Kc6xb7",
            "N/1P6/2kp5/1P6/8/8/8/8 b - - 0 1",
            "N/1k6/3p5/1P6/8/8/8/8 w - - 0 2");
    }

    #[test]
    fn white_castling() {
        legal_moves_include!("8/8/8/8/8/8/8/R3K2R w KQ - 0 1",
            "O-O", "O-O-O");
        board_after_move_is!("O-O",
            "8/8/8/8/8/8/8/R3K2R w KQ - 0 1",
            "8/8/8/8/8/8/8/R31RK1 b - - 1 1");
        board_after_move_is!("O-O-O",
            "8/8/8/8/8/8/8/R3K2R w KQ - 0 1",
            "8/8/8/8/8/8/8/2KR3R b - - 1 1");
        let b = Board::from_fen("8/8/8/8/8/8/8/RN2K2R w KQ - 0 1").unwrap();
        println!("\n{}", b);
        let res: HashSet<Move> = b.legal_moves().unwrap().into_iter().collect();
        assert!(!res.contains(&mv!("O-O-O")));
    }

    #[test]
    fn black_castling() {
        legal_moves_include!("r3k2r/8/8/8/8/8/8/8 b kq - 0 1",
            "O-O", "O-O-O");
        board_after_move_is!("O-O",
            "r3k2r/8/8/8/8/8/8/8 b kq - 0 1",
            "r31rk1/8/8/8/8/8/8/8 w - - 1 2");
        board_after_move_is!("O-O-O",
            "r3k2r/8/8/8/8/8/8/8 b kq - 0 1",
            "2kr3r/8/8/8/8/8/8/8 w - - 1 2");
    }

    #[test]
    #[should_panic]
    fn white_castling_through_threat() {
        // shouldn't be able to castle through threatened square!
        let b = Board::from_fen("8/8/8/8/8/5q3/8/4K2R w KQ - 0 1").unwrap();
        println!("\n{}",b);
        b.make_move(&mv!("O-O")).unwrap();
    }

    #[test]
    #[should_panic]
    fn black_castling_through_threat() {
        // shouldn't be able to castle through threatened square!
        let b = Board::from_fen("4k2r/8/5Q3/8/8/8/8/8 b kq - 0 1").unwrap();
        println!("\n{}",b);
        b.make_move(&mv!("O-O")).unwrap();
    }

    #[test]
    fn checkmate() {
        let b = Board::from_fen("4k3/8/3P4/6Q1/8/8/8/K7 w - - 0 1").unwrap();
        println!("\n{}",b);
        let (mv, _) = b.best_move(1).unwrap();
        assert_eq!(mv, mv!("Qg5-e7"));
    }
}
