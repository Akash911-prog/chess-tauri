use crate::engine::board::Board;

impl Board {
    /// Counts leaf nodes at `depth` by exhaustively making and unmaking
    /// every legal move. Used only to validate move generation + make/unmake
    /// against known-correct reference counts — not a search algorithm.
    fn perft(&mut self, depth: u32) -> u64 {
        if depth == 0 {
            return 1;
        }

        let moves = self.generate_legal_moves();
        let mut nodes = 0;

        for mv in moves {
            self.make_move(mv);
            nodes += self.perft(depth - 1);
            self.undo_move();
        }

        nodes
    }

    /// Same as `perft`, but prints the leaf count contributed by each
    /// individual first move instead of just the total. Compare this
    /// output against a reference engine's divide output for the same
    /// FEN/depth — whichever move's count doesn't match is where the bug
    /// lives. Recurse: set up the position after playing just that move,
    /// run divide again one depth lower, repeat until the mismatch is
    /// isolated to a single move at depth 1.
    fn perft_divide(&mut self, depth: u32) -> u64 {
        let moves = self.generate_legal_moves();
        let mut total = 0;

        for mv in moves {
            self.make_move(mv);
            let nodes = if depth <= 1 { 1 } else { self.perft(depth - 1) };
            self.undo_move();

            println!(
                "{}{}: {}",
                self.index_to_notation(mv.from()),
                self.index_to_notation(mv.to()),
                nodes
            );
            total += nodes;
        }

        println!("total: {}", total);
        total
    }

    fn debug_after_move(&mut self, from: u8, to: u8, depth: u32) {
        let moves = self.generate_legal_moves();

        for mv in moves {
            if mv.from() == from && mv.to() == to {
                self.make_move(mv);

                println!(
                    "\n===== AFTER {}{} =====",
                    self.index_to_notation(from),
                    self.index_to_notation(to)
                );

                self.perft_divide(depth);

                // self.undo_move();
                return;
            }
        }

        panic!("Move not found");
    }
}

fn setup_board(fen: &str) -> Board {
    let mut board = Board::new();
    board.move_gen.generate_moves();
    board.from_fen(fen);
    board.init();
    board
}

#[test]
fn perft_starting_position() {
    let mut board = setup_board("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

    // reference values: https://www.chessprogramming.org/Perft_Results
    assert_eq!(board.perft(1), 20);
    assert_eq!(board.perft(2), 400);
    assert_eq!(board.perft(3), 8_902);
    assert_eq!(board.perft(4), 197_281);
    // depth 5 (4_865_609) is worth running manually once the above pass —
    // slow without move ordering / bulk counting optimizations, don't
    // leave it in the default test run.
}

/// "Kiwipete" — the standard second perft position. Loaded with castling
/// rights on both sides, an en passant opportunity, and pieces set up so
/// pins and discovered checks actually get exercised, unlike the fairly
/// quiet starting position at low depth.
#[test]
fn perft_kiwipete() {
    let mut board =
        setup_board("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");

    assert_eq!(board.perft(1), 48);
    assert_eq!(board.perft(2), 2_039);
    assert_eq!(board.perft(3), 97_862);
    // assert_eq!(board.perft(4), 4_085_603);
    // assert_eq!(board.perft(10), 29_344_805_396_643_919);
    // depth 4 (4_085_603) — manual run only, same reasoning as above.
}

/// Position 3 from the standard perft suite — exercises en passant pins
/// specifically (the "two pawns removed from the same rank" edge case
/// discussed earlier), with minimal other pieces so failures are easy to
/// isolate to that one mechanism.
#[test]
fn perft_position_3() {
    let mut board = setup_board("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");

    assert_eq!(board.perft(1), 14);
    assert_eq!(board.perft(2), 191);
    assert_eq!(board.perft(3), 2_812);
    assert_eq!(board.perft(4), 43_238);
}

/// Position 4 — heavy on promotions, including capturing promotions,
/// which is exactly the bug just fixed in `do_promotion`.
#[test]
fn perft_position_4() {
    let mut board = setup_board("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");

    assert_eq!(board.perft(1), 6);
    assert_eq!(board.perft(2), 264);
    assert_eq!(board.perft(3), 9_467);
}

// #[test]
// fn debug_divide() {
//     let mut board =
//         setup_board("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
//     board.perft_divide(4);

//     board.debug_after_move(8, 24, 3);
//     board.debug_after_move(
//         board.notation_to_index("e8"),
//         board.notation_to_index("g8"),
//         2,
//     );
//     board.debug_after_move(
//         board.notation_to_index("e1"),
//         board.notation_to_index("g1"),
//         1,
//     );
// }
