use crate::engine::{bitboard::BitBoard, types::PieceKind};

impl super::Board {
    pub fn check_for_check(&self) -> CheckInfo {
        let mut info = CheckInfo::new();

        println!("{}", self.player_turn);

        let player = self.player_turn as usize;
        let enemy = (self.player_turn ^ 1) as usize;
        let friendly_occ = self.color_occupency[player];
        let enemy_occ = self.color_occupency[enemy];
        let total_occ = self.total_occupency;

        let king_board = self.pieces[player][PieceKind::King as usize];
        let king_idx = king_board.lsb() as usize;

        println!("{}", self.enemy_attack_mask);

        if (self.enemy_attack_mask & king_board) == 0 {
            return info;
        }
        info.is_check = true;

        let queen_board = self.pieces[enemy][PieceKind::Queen as usize];

        // (attack pattern from king square, enemy pieces that pattern can hit, kind if matched)
        let checkers_by_kind = [
            (
                self.move_gen
                    .get_knight_moves(king_idx, friendly_occ)
                    .unwrap_or(BitBoard(0))
                    & self.pieces[enemy][PieceKind::Knight as usize],
                PieceKind::Knight,
            ),
            (
                self.move_gen
                    .get_pawn_attacks(king_idx, self.player_turn, total_occ, enemy_occ, true)
                    .unwrap_or(BitBoard(0))
                    & self.pieces[enemy][PieceKind::Pawn as usize],
                PieceKind::Pawn,
            ),
            (
                self.move_gen
                    .get_rook_moves(king_idx, friendly_occ)
                    .unwrap_or(BitBoard(0))
                    & (self.pieces[enemy][PieceKind::Rook as usize] | queen_board),
                PieceKind::Rook,
            ),
            (
                self.move_gen
                    .get_bishop_moves(king_idx, friendly_occ)
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

    pub fn validate_move_with_check(&self, from: u8, to: u8) -> bool {
        let check_info = self.check_for_check();
        true
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
