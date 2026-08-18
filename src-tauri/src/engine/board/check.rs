use crate::engine::{bitboard::BitBoard, types::PieceKind};

impl super::Board {
    /// Checks if the current player is in check.
    ///
    /// This function analyzes the current state of the game board and determines
    /// whether the current player's king is under attack by an enemy piece.
    ///
    /// # Returns
    ///
    /// Returns a `CheckInfo` struct containing information about the check:
    /// - `is_check`: A boolean indicating whether the current player is in check.
    /// - `check_square`: An array of squares that can be used to escape the check.
    /// - `piece_idx`: An array of piece kinds that are responsible for the check.
    /// - `count`: The number of squares in the `check_square` array.
    ///
    /// # Note
    ///
    /// This function assumes that the board is in a valid state and that the
    /// current player's turn is correctly set.
    pub fn check_for_check(&self) -> CheckInfo {
        let mut info = CheckInfo::new();

        let player = self.player_turn as usize;
        let enemy = (self.player_turn ^ 1) as usize;
        let friendly_occ = self.color_occupency[player];
        let enemy_occ = self.color_occupency[enemy];
        let total_occ = self.total_occupency;

        let king_board = self.pieces[player][PieceKind::King as usize];
        let king_idx = king_board.lsb() as usize;

        if (self.attack_mask[enemy] & king_board) == 0 {
            return info;
        }

        info.is_check = true;

        let queen_board = self.pieces[enemy][PieceKind::Queen as usize];

        // (attack pattern from king square, enemy pieces that pattern can hit, kind if matched)
        let checkers_by_kind = [
            (
                self.move_gen
                    .get_knight_moves(king_idx, friendly_occ, false)
                    .unwrap_or(BitBoard(0))
                    & self.pieces[enemy][PieceKind::Knight as usize],
                PieceKind::Knight,
            ),
            (
                self.move_gen
                    .get_pawn_attacks(
                        king_idx,
                        self.player_turn,
                        total_occ,
                        enemy_occ,
                        true,
                        self.en_passant_square,
                    )
                    .unwrap_or(BitBoard(0))
                    & self.pieces[enemy][PieceKind::Pawn as usize],
                PieceKind::Pawn,
            ),
            (
                self.move_gen
                    .get_rook_moves(
                        king_idx,
                        total_occ,
                        friendly_occ,
                        self.player_turn,
                        true,
                        &self.kings,
                    )
                    .unwrap_or(BitBoard(0))
                    & (self.pieces[enemy][PieceKind::Rook as usize] | queen_board),
                PieceKind::Rook,
            ),
            (
                self.move_gen
                    .get_bishop_moves(
                        king_idx,
                        total_occ,
                        friendly_occ,
                        self.player_turn,
                        true,
                        &self.kings,
                    )
                    .unwrap_or(BitBoard(0))
                    & (self.pieces[enemy][PieceKind::Bishop as usize] | queen_board),
                PieceKind::Bishop,
            ),
        ];

        let mut checkers_count = 0;
        'outer: for (mut checkers, fallback_kind) in checkers_by_kind {
            while let Some(sq) = checkers.pop_lsb() {
                let checker_bit = BitBoard(1u64 << sq);
                let kind = if (checker_bit & queen_board) != 0 {
                    PieceKind::Queen
                } else {
                    fallback_kind
                };

                info.check_square[checkers_count] = sq as u8;
                info.piece_idx[checkers_count] = kind as u8;
                checkers_count += 1;

                if checkers_count >= 2 {
                    break 'outer;
                }
            }
        }
        info.count = checkers_count as u8;

        info
    }

    pub fn validate_move_with_check(
        &self,
        to: u8,
        check_info: &CheckInfo,
        piece_type: PieceKind,
    ) -> bool {
        if (check_info.count >= 2) && (piece_type != PieceKind::King) {
            return false;
        }

        if check_info.check_square[0] == to {
            return true;
        }

        if check_info.count == 0 {
            return false;
        }

        let attack_piece = PieceKind::from_idx(check_info.piece_idx[0] as usize);

        match attack_piece {
            PieceKind::Bishop => {
                let occupied = BitBoard(1u64 << to);
                let attack_map = self.move_gen.gen_bishop_attacks(
                    check_info.check_square[0] as usize,
                    occupied,
                    self.player_turn,
                    false,
                    &self.kings,
                );

                if (attack_map & self.pieces[self.player_turn as usize][PieceKind::King as usize])
                    == 0
                {
                    return true;
                }
                return false;
            }
            PieceKind::Rook => {
                let occupied = BitBoard(1u64 << to);
                let attack_map = self.move_gen.gen_rook_attacks(
                    check_info.check_square[0] as usize,
                    occupied,
                    self.player_turn,
                    false,
                    &self.kings,
                );

                if (attack_map & self.pieces[self.player_turn as usize][PieceKind::King as usize])
                    == 0
                {
                    return true;
                }
                return false;
            }
            PieceKind::Queen => {
                let occupied = BitBoard(1u64 << to);
                let attack_map = self.move_gen.gen_queen_attacks(
                    check_info.check_square[0] as usize,
                    occupied,
                    self.player_turn,
                    false,
                    &self.kings,
                );

                if (attack_map & self.pieces[self.player_turn as usize][PieceKind::King as usize])
                    == 0
                {
                    return true;
                }
                return false;
            }
            PieceKind::Knight | PieceKind::Pawn => {
                if piece_type != PieceKind::King {
                    return false;
                } else {
                    return true;
                }
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CheckInfo {
    pub is_check: bool,
    pub check_square: [u8; 2],
    pub piece_idx: [u8; 2],
    pub count: u8,
}

impl CheckInfo {
    fn new() -> CheckInfo {
        CheckInfo {
            is_check: false,
            check_square: [64; 2],
            piece_idx: [6; 2],
            count: 0,
        }
    }
}
