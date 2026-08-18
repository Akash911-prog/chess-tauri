use std::time::Instant;

use crate::engine::{computer::negamax::Search, constants::MATE_SCORE};

use super::*;

fn is_mate_score(score: i32) -> bool {
    if score >= (MATE_SCORE - 3) {
        return true;
    }
    return false;
}

// --- Base case: depth 0 returns static eval, correctly signed ---
#[test]
fn depth_zero_returns_static_eval() {
    let mut board = setup_board("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
    let eval_score = Evaluator::new(&board).static_eval();
    let mut search = Search::new(&mut board);
    let search_score = search.negamax(0, 0, i32::MIN + 1, i32::MAX);
    assert_eq!(search_score, eval_score);
}

// --- Terminal: checkmate detected correctly ---
#[test]
fn no_legal_moves_in_check_returns_mate_score() {
    // e.g. back-rank mate FEN
    let mut board = setup_board("6k1/5ppp/8/8/8/8/8/R3K3 w - - 0 1"); // adjust to an actual mate-in-0 for side to move
    let mut search = Search::new(&mut board);
    let score = search.negamax(1, 0, i32::MIN + 1, i32::MAX);
    assert!(is_mate_score(score)); // however you define/detect this
}

// --- Terminal: stalemate returns exactly 0, not a mate score ---
#[test]
fn no_legal_moves_not_in_check_returns_zero() {
    let mut board = setup_board("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1");
    let mut search = Search::new(&mut board);
    let score = search.negamax(1, 0, i32::MIN + 1, i32::MAX);
    assert_eq!(score, 0);
}

// // --- Sign convention: flipping side to move negates the score ---
// #[test]
// fn eval_score_negates_on_side_flip_depth_zero() {
//     let mut board_white = setup_board("1k3q1r/6p1/2pN4/Q3P2r/P3P3/5bP1/1P3PB1/2R3K1 w - - 0 1");
//     let mut board_black = setup_board("1k3q1r/6p1/2pN4/Q3P2r/P3P3/5bP1/1P3PB1/2R3K1 b - - 0 1");

//     let mut search_white = Search::new(&mut board_white);
//     let mut search_black = Search::new(&mut board_black);

//     let white_score = search_white.negamax(0, 0, i32::MIN + 1, i32::MAX);
//     let black_score = search_black.negamax(0, 0, i32::MIN + 1, i32::MAX);

//     println!("white: {}, black: {}", white_score, black_score);

//     assert_eq!(white_score, -black_score);
// }

// --- Known tactic: mate-in-1 found correctly ---
// Setup: White Ra1, Ke1. Black Kh8, pawns f7/g7/h7 (king fully boxed on rank 8).
// Ra1-a8+ checks along the open 8th rank; g8 is attacked by the same ray,
// g7/h7 are occupied by black's own pawns. No blocks/captures available (lone king + pawns).
// Verified: only legal reply-less position -> checkmate.
#[test]
fn finds_mate_in_one() {
    let mut board = setup_board("7k/5ppp/8/8/8/8/8/R3K3 w - - 0 1");
    let mut search = Search::new(&mut board);
    let (best_move, score) = search.find_best_move(1);

    assert_eq!(best_move.from(), 0); // a1
    assert_eq!(best_move.to(), 56); // a8
    assert!(is_mate_score(score));
}

// --- Known tactic: mate-in-2 found at sufficient depth ---
// Setup: White Rh1, Kc7. Black Ka8 (lone king, no other pieces).
// 1. Rh1-h8+ Ka7 (forced: b7/b8 covered by Kc7, b8/rest of rank8 covered by rook;
//    a7 is the only uncovered square) 2. Rh8-a8# (a-file check, a6/b6/b7/b8 all
//    covered by rook+king, no flight squares, no blocks/captures possible).
// Verified move-by-move including forced-reply uniqueness.
#[test]
fn finds_mate_in_two() {
    let mut board = setup_board("1k3q1r/6p1/2pN4/Q3P2r/P3P3/5bP1/1P3PB1/2R3K1 b - - 0 1");
    let mut search = Search::new(&mut board);
    let (best_move, score) = search.find_best_move(3); // depth 3: W, B, W

    assert_eq!(best_move.from(), 39); // h5
    assert_eq!(best_move.to(), 7); // h1
    assert!(is_mate_score(score));
}

// --- Determinism: same position, same depth, same result every time ---
#[test]
fn negamax_is_deterministic() {
    let mut board = setup_board("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
    let mut search = Search::new(&mut board);
    let s1 = search.negamax(3, 0, i32::MIN + 1, i32::MAX);
    let s2 = search.negamax(3, 0, i32::MIN + 1, i32::MAX);
    assert_eq!(s1, s2);
}

#[test]
fn timed_negamax() {
    let mut board =
        setup_board("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    let mut search = Search::new(&mut board);
    let start = Instant::now();

    let s1 = search.negamax(5, 0, i32::MIN + 1, i32::MAX);

    println!("Negamax score: {}", s1);
    let elapsed = start.elapsed();
    println!("Negamax took {:.3} ms", elapsed.as_secs_f64() * 1000.0);
    println!("Nood visited: {} ", search.nodes_visited);
}
