use crate::engine::{bitboard::BitBoard, types::PieceKind};

impl super::Board {
    fn check_for_check(&self) -> CheckInfo {
        let mut info = CheckInfo::new();

        let player = self.player_turn as usize;
        let enemy = (self.player_turn ^ 1) as usize;
        let friendly_occ = self.color_occupency[player];
        let enemy_occ = self.color_occupency[enemy];
        let total_occ = self.total_occupency;

        let king_board = self.pieces[player][PieceKind::King as usize];
        let king_idx = king_board.lsb() as usize;

        if (self.enemy_attack_mask & king_board) == 0 {
            return info;
        }
        info.is_check = true;

        let mut checkers_count = 0;
        let knight_checker = self
            .move_gen
            .get_knight_moves(king_idx, friendly_occ)
            .unwrap_or(BitBoard(0));
        let pawn_checker = self
            .move_gen
            .get_pawn_attacks(king_idx, self.player_turn, total_occ, enemy_occ, true)
            .unwrap_or(BitBoard(0));
        let rook_checker = self
            .move_gen
            .get_rook_moves(king_idx, friendly_occ)
            .unwrap_or(BitBoard(0));
        let bishop_checker = self
            .move_gen
            .get_bishop_moves(king_idx, friendly_occ)
            .unwrap_or(BitBoard(0));

        let knight_checkers = knight_checker & self.pieces[enemy][PieceKind::Knight as usize];
        let pawn_checkers = pawn_checker & self.pieces[enemy][PieceKind::Pawn as usize];
        let rook_checkers = rook_checker
            & (self.pieces[enemy][PieceKind::Rook as usize]
                | self.pieces[enemy][PieceKind::Queen as usize]);
        let bishop_checkers = bishop_checker
            & (self.pieces[enemy][PieceKind::Bishop as usize]
                | self.pieces[enemy][PieceKind::Queen as usize]);

        let queen_board = self.pieces[enemy][PieceKind::Queen as usize];

        let mut all_checkers = knight_checkers | pawn_checkers | rook_checkers | bishop_checkers;
        let mut checker = all_checkers.pop_lsb();
        while let Some(sq) = checker {
            if checkers_count >= 2 {
                break;
            }
            let checker_idx = sq as usize;
            let checker_bit = BitBoard(1u64 << checker_idx);

            let kind = if (checker_bit & queen_board) != 0 {
                PieceKind::Queen
            } else if (checker_bit & knight_checkers) != 0 {
                PieceKind::Knight
            } else if (checker_bit & pawn_checkers) != 0 {
                PieceKind::Pawn
            } else if (checker_bit & rook_checkers) != 0 {
                PieceKind::Rook
            } else {
                PieceKind::Bishop
            };

            info.check_square[checkers_count] = checker_idx as u8;
            info.piece_idx[checkers_count] = kind as u8;
            checkers_count += 1;
            checker = all_checkers.pop_lsb();
        }

        info.count = checkers_count as u8;

        info
    }

    pub fn validate_move_with_check(&self, from: u8, to: u8) -> bool {
        let check_info = self.check_for_check();
        true
    }
}

struct CheckInfo {
    is_check: bool,
    check_square: [u8; 2],
    piece_idx: [u8; 2],
    count: u8,
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
