#[cfg(test)]
pub mod perft;

use super::*;
use crate::engine::{
    board::Board,
    movegen::{Move, MoveFlag},
    types::PieceKind,
};

#[test]
fn legal_test() {
    let game = Game::new();

    {
        let mut board = game.board.lock().unwrap();
        board.move_gen.generate_moves();
        board.init();
    }

    let legal_moves = game.board.lock().unwrap().get_all_legal_moves(0, false);

    // legal_moves.iter().enumerate().for_each(|(i, bb)| {
    //     println!("{}: {}", i, bb);
    // });

    let total: u32 = legal_moves.iter().map(|bb| bb.count()).sum();
    assert_eq!(
        total, 20,
        "starting position should have exactly 20 legal moves for white, got {total}"
    );
}

#[test]
fn check_test() {
    let game = Game::new();

    let mut board = game.board.lock().unwrap();
    board.move_gen.generate_moves();
    board.init();

    board.from_fen("8/8/8/8/8/6n1/8/r6K w - - 0 1");

    let check_info = board.check_for_check();

    assert_eq!(true, check_info.is_check);
}

fn setup_board(fen: &str) -> Board {
    let mut board = Board::new();
    board.from_fen(fen);
    board.move_gen.generate_moves();
    board.init();
    board
}

// piece byte: low nibble = moving piece, high nibble = captured piece (>5 = none)
fn packed_piece(piece: PieceKind, captured: Option<PieceKind>) -> u8 {
    let cap = captured.map(|c| c as u8).unwrap_or(0x0F);
    (piece as u8 & 0x0F) | (cap << 4)
}

fn quiet_move(from: u8, to: u8, piece: PieceKind) -> Move {
    let mask = (from as u16) | ((to as u16) << 6);
    Move::new(mask, packed_piece(piece, None))
}

fn capture_move(from: u8, to: u8, piece: PieceKind, captured: PieceKind) -> Move {
    let mask = (from as u16) | ((to as u16) << 6);
    Move::new(mask, packed_piece(piece, Some(captured)))
}

#[test]
fn en_passant_capture() {
    let mut board = setup_board("rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3");
    assert_eq!(board.en_passant_square, 45); // f6

    let mask = (36u16) | (45u16 << 6) | MoveFlag::EpCapture.bits();
    let mv = Move::new(mask, packed_piece(PieceKind::Pawn, Some(PieceKind::Pawn)));
    board.make_move(mv);

    assert_eq!(board.pieces[1][PieceKind::Pawn as usize] & (1u64 << 37), 0); // f5 cleared
    assert_ne!(board.pieces[0][PieceKind::Pawn as usize] & (1u64 << 45), 0);
    // f6 occupied
}

#[test]
fn white_kingside_castle() {
    let mut board = setup_board("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");

    let mask = (4u16) | (6u16 << 6) | MoveFlag::KingCastle.bits();
    let mv = Move::new(mask, packed_piece(PieceKind::King, None));
    board.make_move(mv);

    assert_ne!(board.pieces[0][PieceKind::King as usize] & (1u64 << 6), 0);
    assert_ne!(board.pieces[0][PieceKind::Rook as usize] & (1u64 << 5), 0);
    assert_eq!(board.pieces[0][PieceKind::Rook as usize] & (1u64 << 7), 0);
    assert_eq!(board.castling_rights & 0x0C, 0); // K and Q gone
}

#[test]
fn white_queenside_castle() {
    let mut board = setup_board("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");

    let mask = (4u16) | (2u16 << 6) | MoveFlag::QueenCastle.bits();
    let mv = Move::new(mask, packed_piece(PieceKind::King, None));
    board.make_move(mv);

    assert_ne!(board.pieces[0][PieceKind::King as usize] & (1u64 << 2), 0);
    assert_ne!(board.pieces[0][PieceKind::Rook as usize] & (1u64 << 3), 0);
    assert_eq!(board.pieces[0][PieceKind::Rook as usize] & (1u64 << 0), 0);
    assert_eq!(board.castling_rights & 0x0C, 0);
}

#[test]
fn black_kingside_castle() {
    let mut board = setup_board("r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1");

    let mask = (60u16) | (62u16 << 6) | MoveFlag::KingCastle.bits();
    let mv = Move::new(mask, packed_piece(PieceKind::King, None));
    board.make_move(mv);

    assert_ne!(board.pieces[1][PieceKind::King as usize] & (1u64 << 62), 0);
    assert_ne!(board.pieces[1][PieceKind::Rook as usize] & (1u64 << 61), 0);
    assert_eq!(board.pieces[1][PieceKind::Rook as usize] & (1u64 << 63), 0);
    assert_eq!(board.castling_rights & 0x03, 0);
}

#[test]
fn single_check_resolved_by_capture() {
    let mut board = setup_board("4k3/8/8/8/8/8/4r3/4K3 w - - 0 1");

    let info = board.check_for_check();
    assert!(info.is_check);
    assert_eq!(info.count, 1);
    assert_eq!(info.check_square[0], 12); // e2
    assert_eq!(info.piece_idx[0], PieceKind::Rook as u8);

    let mv = capture_move(4, 12, PieceKind::King, PieceKind::Rook);
    board.make_move(mv);

    let info_after = board.check_for_check();
    assert!(!info_after.is_check);
}

#[test]
fn single_check_resolved_by_block() {
    let mut board = setup_board("4k3/8/8/8/8/1R6/8/r3K3 w - - 0 1");

    let info = board.check_for_check();
    assert!(info.is_check);
    assert_eq!(info.count, 1);
    assert_eq!(info.check_square[0], 0); // a1
    assert_eq!(info.piece_idx[0], PieceKind::Rook as u8);

    let mv = quiet_move(17, 1, PieceKind::Rook); // b3 -> b1
    board.make_move(mv);

    let info_after = board.check_for_check();
    assert!(!info_after.is_check);
}

#[test]
fn double_check_king_must_move() {
    let board = setup_board("8/8/8/8/8/6n1/8/r6K w - - 0 1");

    let info = board.check_for_check();
    assert!(info.is_check);
    assert_eq!(info.count, 2);
}

// #[test]
// fn promotion_produces_correct_piece_type() {
//     for (promo_notation, expected) in [
//         ("q", PieceKind::Queen),
//         ("r", PieceKind::Rook),
//         ("b", PieceKind::Bishop),
//         ("n", PieceKind::Knight),
//     ] {
//         let mut board = setup_board("8/P7/8/8/8/8/8/4K2k w - - 0 1");
//         let mv = /* build a1-a8 pawn move with the corresponding PromoX flag */;
//         board.make_move(mv);

//         for kind in [PieceKind::Queen, PieceKind::Rook, PieceKind::Bishop, PieceKind::Knight] {
//             let bit = board.pieces[0][kind as usize] & (1u64 << 56);
//             if kind == expected {
//                 assert_ne!(bit, 0, "promoting to {promo_notation} should set the {kind:?} bitboard");
//             } else {
//                 assert_eq!(bit, 0, "promoting to {promo_notation} should NOT touch the {kind:?} bitboard");
//             }
//         }
//     }
// }
