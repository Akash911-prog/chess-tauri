use super::*;
use crate::engine::{
    bitboard::BitBoard,
    movegen::{Move, MoveFlag},
};

fn setup_board(fen: &str) -> Board {
    let mut board = Board::new();
    board.from_fen(fen);
    board.move_gen.generate_moves();
    board.init();
    board
}

/// Packs a Move by hand from raw components, bypassing move generation,
/// so each test controls exactly which flag/piece/capture combination
/// gets exercised.
fn raw_move(from: u8, to: u8, flag_bits: u16, piece: u8, captured: u8) -> Move {
    let mask = (from as u16) | ((to as u16) << 6) | flag_bits;
    let piece_byte = (piece & 0x0F) | ((captured & 0x0F) << 4);
    Move::new(mask, piece_byte)
}

// --- Case 1: quiet move, no capture ---
// White knight b1 -> c3. b1 = rank0*8+file1 = 1. c3 = rank2*8+file2 = 18.
#[test]
fn incremental_occupancy_matches_full_recompute_quiet_move() {
    let mv = raw_move(
        1,
        18,
        MoveFlag::Quiet.bits(),
        1, /* Knight */
        6, /* no capture */
    );

    let mut board_full = setup_board("4k3/8/8/8/8/8/8/1N2K3 w - - 0 1");
    board_full.pieces[0][1] ^= (1u64 << mv.from()) | (1u64 << mv.to()); // move the knight
    board_full.init_state();

    let mut board_incremental = setup_board("4k3/8/8/8/8/8/8/1N2K3 w - - 0 1");
    board_incremental.pieces[0][1] ^= (1u64 << mv.from()) | (1u64 << mv.to());
    board_incremental.update_state_incremental(mv, 0);

    assert_eq!(
        board_full.color_occupency,
        board_incremental.color_occupency
    );
    assert_eq!(
        board_full.total_occupency,
        board_incremental.total_occupency
    );
}

// --- Case 2: normal capture ---
// White pawn e5 captures black pawn d6. e5 = rank4*8+file4 = 36. d6 = rank5*8+file3 = 43.
#[test]
fn incremental_occupancy_matches_full_recompute_capture() {
    let mv = raw_move(
        36,
        43,
        MoveFlag::Capture.bits(),
        0, /* Pawn */
        0, /* captured Pawn */
    );

    let mut board_full = setup_board("4k3/8/3p4/4P3/8/8/8/4K3 w - - 0 1");
    board_full.pieces[0][0] ^= (1u64 << mv.from()) | (1u64 << mv.to()); // white pawn moves
    board_full.pieces[1][0] ^= 1u64 << mv.to(); // black pawn removed
    board_full.init_state();

    let mut board_incremental = setup_board("4k3/8/3p4/4P3/8/8/8/4K3 w - - 0 1");
    board_incremental.pieces[0][0] ^= (1u64 << mv.from()) | (1u64 << mv.to());
    board_incremental.pieces[1][0] ^= 1u64 << mv.to();
    board_incremental.update_state_incremental(mv, 0);

    assert_eq!(
        board_full.color_occupency,
        board_incremental.color_occupency
    );
    assert_eq!(
        board_full.total_occupency,
        board_incremental.total_occupency
    );
}

// --- Case 3: en passant capture ---
// White pawn e5 x d6 e.p., capturing black pawn actually sitting on d5.
// e5 = 36, d6 = 43, d5 = rank4*8+file3 = 35.
#[test]
fn incremental_occupancy_matches_full_recompute_en_passant() {
    let mv = raw_move(36, 43, MoveFlag::EpCapture.bits(), 0 /* Pawn */, 0);

    let mut board_full = setup_board("4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1");
    board_full.pieces[0][0] ^= (1u64 << mv.from()) | (1u64 << mv.to()); // white pawn moves
    board_full.pieces[1][0] ^= 1u64 << 35; // black pawn on d5 removed
    board_full.init_state();

    let mut board_incremental = setup_board("4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1");
    board_incremental.pieces[0][0] ^= (1u64 << mv.from()) | (1u64 << mv.to());
    board_incremental.pieces[1][0] ^= 1u64 << 35;
    board_incremental.update_state_incremental(mv, 0);

    assert_eq!(
        board_full.color_occupency,
        board_incremental.color_occupency
    );
    assert_eq!(
        board_full.total_occupency,
        board_incremental.total_occupency
    );
}

// --- Case 4: kingside castle ---
// White king e1 -> g1 (4 -> 6), rook h1 -> f1 (7 -> 5), per do_castle's constants.
#[test]
fn incremental_occupancy_matches_full_recompute_castle() {
    let mv = raw_move(
        4,
        6,
        MoveFlag::KingCastle.bits(),
        5, /* King */
        6, /* no capture */
    );

    let mut board_full = setup_board("4k3/8/8/8/8/8/8/4K2R w KQ - 0 1");
    board_full.pieces[0][5] ^= (1u64 << mv.from()) | (1u64 << mv.to()); // king
    board_full.pieces[0][3] ^= (1u64 << 7) | (1u64 << 5); // rook h1 -> f1
    board_full.init_state();

    let mut board_incremental = setup_board("4k3/8/8/8/8/8/8/4K2R w KQ - 0 1");
    board_incremental.pieces[0][5] ^= (1u64 << mv.from()) | (1u64 << mv.to());
    board_incremental.pieces[0][3] ^= (1u64 << 7) | (1u64 << 5);
    board_incremental.update_state_incremental(mv, 0);

    assert_eq!(
        board_full.color_occupency,
        board_incremental.color_occupency
    );
    assert_eq!(
        board_full.total_occupency,
        board_incremental.total_occupency
    );
}

#[test]
fn attack_by_type_fold_matches_old_attack_mask() {
    let mut board =
        setup_board("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    board.update_attack_mask();
    assert_eq!(
        board.attack_mask[0],
        board.attack_by_type[0]
            .iter()
            .fold(BitBoard::EMPTY, |a, b| a | *b)
    );
    assert_eq!(
        board.attack_mask[1],
        board.attack_by_type[1]
            .iter()
            .fold(BitBoard::EMPTY, |a, b| a | *b)
    );
}
