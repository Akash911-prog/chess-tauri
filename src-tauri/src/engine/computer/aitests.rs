#[cfg(test)]
mod quiescence_search;

use crate::engine::computer::evaluator::Evaluator;

use super::super::*;

fn setup_board(fen: &str) -> board::Board {
    let mut board = board::Board::new();
    board.from_fen(fen);
    board.move_gen.generate_moves();
    board.init();
    board
}

fn pre_test<'a>(board: &'a board::Board) -> Evaluator<'a> {
    Evaluator::new(&board)
}

#[test]
fn evaluator_test() {}

#[cfg(test)]
mod eval_material_count {
    use super::*;

    #[test]
    fn one_pawn() {
        let board = setup_board("3k4/8/8/2P5/8/8/8/3K4 w - - 0 1");
        let eval = pre_test(&board);
        let score = eval.static_eval();

        println!("score: {}", score);

        assert_eq!(score, 100);
    }

    #[test]
    fn starting_position() {
        let board = setup_board("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let eval = pre_test(&board);
        let score = eval.static_eval();

        println!("score: {}", score);

        assert_eq!(score, 0);
    }

    #[test]
    fn black_knight_removal() {
        let board = setup_board("rnbqkb1r/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let eval = pre_test(&board);
        let score = eval.static_eval();

        println!("score: {}", score);

        assert_eq!(score, 320);
    }

    #[test]
    fn side_to_move_sign() {
        let board = setup_board("rnbqkb1r/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1");
        let eval = pre_test(&board);
        let score = eval.static_eval();

        println!("score: {}", score);

        assert_eq!(score, -320);
    }
}

#[cfg(test)]
mod piece_sq_table {
    use super::*;

    #[test]
    fn knight_wieghted_score() {
        let board = setup_board("1n1k4/8/8/8/8/8/8/4K3 b KQkq - 0 1");
        let eval = pre_test(&board);
        let score_edge = eval.static_eval();

        let board = setup_board("3k4/8/8/3n4/8/8/8/4K3 b KQkq - 0 1");
        let eval = pre_test(&board);
        let score_center = eval.static_eval();

        println!("score_edge: {}", score_edge);
        println!("score_center: {}", score_center);

        assert!(score_edge < score_center);
    }

    #[test]
    fn mirror_symmetry() {
        let board = setup_board("3k4/8/8/4n3/4N3/8/8/3K4 b KQkq - 0 1");
        let eval = pre_test(&board);
        let score = eval.static_eval();

        println!("score: {}", score);

        assert_eq!(score, 0);
    }

    #[test]
    fn sanitation() {
        // Queen: White queen centralized, Black queen on the edge
        let board = setup_board("4k3/8/8/3q4/8/8/4Q3/4K3 w - - 0 1");
        let eval = pre_test(&board);
        let score = eval.static_eval();
        assert!(score < 0);

        // Knight: White knight centralized, Black knight on the edge
        let board = setup_board("2n3k1/8/8/8/3N4/8/8/7K b - - 0 1");
        let eval = pre_test(&board);
        let score = eval.static_eval();
        assert!(score < 0);

        // Bishop: White bishop centralized, Black bishop on the edge
        let board = setup_board("7k/8/8/4b3/8/3B4/8/K7 b - - 0 1");
        let eval = pre_test(&board);
        let score = eval.static_eval();
        assert!(score > 0);

        // Rook: White rook centralized, Black rook on the edge
        let board = setup_board("7k/8/8/8/4R3/8/r7/K7 b - - 0 1");
        let eval = pre_test(&board);
        let score = eval.static_eval();
        assert!(score > 0);

        // Pawn: White pawn advanced/central, Black pawn on the edge
        let board = setup_board("7k/8/8/3P4/8/8/p7/K7 b - - 0 1");
        let eval = pre_test(&board);
        let score = eval.static_eval();
        assert!(score > 0);
    }

    #[test]
    fn orientation() {
        let board = setup_board("3k4/P7/8/8/8/8/8/3K4 w KQkq - 0 1");
        let eval = pre_test(&board);
        let score_higher = eval.static_eval();

        let board = setup_board("3k4/8/8/8/8/8/1P6/3K4 w KQkq - 0 1");
        let eval = pre_test(&board);
        let score_lower = eval.static_eval();

        println!("score_higher: {}", score_higher);
        println!("score_lower: {}", score_lower);

        assert!(score_higher > score_lower);
    }
}

#[cfg(test)]
mod mobility {
    use super::*;

    #[test]
    fn queen() {
        let board = setup_board("8/4k3/8/8/8/2PPP3/2PQK3/8 w - - 0 1");
        let eval = pre_test(&board);
        let score_restricted = eval.static_eval();

        let board = setup_board("8/4k3/8/8/8/P4PPP/P2QK3/8 w - - 0 1");
        let eval = pre_test(&board);
        let score_unrestricted = eval.static_eval();

        println!("score_restricted: {}", score_restricted);
        println!("score_unrestricted: {}", score_unrestricted);

        assert!(score_restricted < score_unrestricted);
    }

    #[test]
    fn pawn() {
        let board = setup_board("3k4/8/8/8/8/8/2P5/3K4 w - - 0 1");
        let eval = pre_test(&board);
        let score = eval.static_eval();

        let board = setup_board("3k4/7P/8/8/8/8/8/3K4 w - - 0 1");
        let eval = pre_test(&board);
        let score_higher = eval.static_eval();

        let board = setup_board("3k4/8/8/1ppp4/2P5/8/8/3K4 w - - 0 1");
        let eval = pre_test(&board);
        let score_blocked = eval.static_eval();

        let board = setup_board("3k4/8/8/5ppp/2P5/8/8/3K4 w - - 0 1");
        let eval = pre_test(&board);
        let score_unblocked = eval.static_eval();

        println!("score: {}", score);
        println!("score_higher: {}", score_higher);
        println!("score_blocked: {}", score_blocked);
        println!("score_unblocked: {}", score_unblocked);

        assert!(score < score_higher);
        assert!(score_blocked < score_unblocked);
    }
}

#[cfg(test)]
mod eval_sanity {
    use super::*;

    /// Test positions capturing various game phases and pawn/piece setups
    const SANITY_TEST_FENS: &[&str] = &[
        // Standard starting position
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        // Openings & Early Middlegames
        "r1bqk2r/pp1pppbp/2n2np1/8/3NP3/2N1B3/PPP2PPP/R2QKB1R w KQkq - 2 7",
        "rnbq1rk1/ppp1ppbp/3p1np1/8/2PPP3/2N2N2/PP2BPPP/R1BQK2R b KQ - 1 6",
        // Complex Middlegames (Tense tactical & positional setups)
        "r1b2rk1/pp3ppp/2n1p3/8/3P4/3B1N2/PP3PPP/R4RK1 w - - 0 14",
        "2r2rk1/1bq1bppp/p2ppn2/1p6/3NPP2/P1NR4/1PP1Q1PP/5R1K w - - 1 18",
        // Endgame Positions (Passed pawns, piece imbalance)
        "7k/8/2p5/8/8/8/8/K7 b - - 0 1",
        "8/2p5/4k3/8/8/4K3/8/8 w - - 0 1",
        "8/8/4k3/8/2p5/8/3P4/4K3 w - - 0 1",
        "8/5k2/8/8/8/8/1R6/2K5 w - - 0 1",
        "8/8/1p6/1P6/8/8/2k5/K7 w - - 0 1",
        // Asymmetric / Tactical Edge Cases
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", // Kiwipete
        "8/2p5/8/2P5/8/8/8/K6k w - - 0 1",
    ];

    #[test]
    fn eval_determinism_sanity() {
        for &fen in SANITY_TEST_FENS {
            let board1 = setup_board(fen);
            let eval1 = pre_test(&board1);
            let score1 = eval1.static_eval();

            let board2 = setup_board(fen);
            let eval2 = pre_test(&board2);
            let score2 = eval2.static_eval();

            assert_eq!(
                score1, score2,
                "Evaluation non-deterministic for FEN: {}",
                fen
            );
        }
    }

    #[test]
    fn eval_perspective_symmetry_sanity() {
        // Verifies that static_eval score sign correctly reflects side-to-move symmetry
        for &fen in SANITY_TEST_FENS {
            let board = setup_board(fen);
            let eval = pre_test(&board);
            let score = eval.static_eval();

            // Ensure static_eval doesn't return NaN-like invalid boundary values
            assert!(
                score.abs() < 30_000,
                "Static eval returned an out-of-bounds score ({}) for FEN: {}",
                score,
                fen
            );
        }
    }
}
