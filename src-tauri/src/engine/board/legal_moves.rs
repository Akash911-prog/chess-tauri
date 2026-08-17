use crate::engine::{
    movegen::{Move, MoveFlag},
    types::PieceKind,
};

impl super::Board {
    /// Returns every fully legal move for the side to move.
    ///
    /// This is the single source of truth move generation should route
    /// through — pseudo-legal generation filtered by `validate_move`
    /// (pins + check evasion) for non-king pieces, and `validate_king_move`
    /// (attack-mask safety + castling) for the king.
    pub fn generate_legal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();

        let color = self.player_turn as usize;
        let friendly = self.color_occupency[color];
        let enemy = self.color_occupency[color ^ 1];
        let occupied = self.total_occupency;

        for piece_idx in 0..6 {
            let piece_type = PieceKind::from_idx(piece_idx);
            let mut piece_board = self.pieces[color][piece_idx];

            while let Some(from) = piece_board.pop_lsb() {
                let from = from as u8;

                if piece_type == PieceKind::King {
                    self.collect_king_moves(from, &mut moves);
                    continue;
                }

                let Some(mut destinations) = self.move_gen.get_legal_moves_by_piece(
                    piece_type,
                    occupied,
                    from as usize,
                    self.player_turn,
                    enemy,
                    friendly,
                    false,
                    self.en_passant_square,
                    &self.kings,
                ) else {
                    continue;
                };

                while let Some(to) = destinations.pop_lsb() {
                    let to = to as u8;
                    if !self.validate_move(piece_type, from, to) {
                        continue;
                    }
                    self.push_pseudo_legal_move(piece_idx, from, to, &mut moves);
                }
            }
        }

        moves
    }

    fn collect_king_moves(&self, from: u8, moves: &mut Vec<Move>) {
        let friendly = self.color_occupency[self.player_turn as usize];

        // adjacent squares
        if let Some(mut candidates) = self.move_gen.get_king_attacks(from as usize, friendly) {
            while let Some(to) = candidates.pop_lsb() {
                self.try_push_king_move(from, to as u8, moves);
            }
        }

        // castle destinations are two squares away — not in the king attack
        // table, so they need to be tested explicitly.
        let castle_targets: [(u8, u8); 2] = if self.player_turn == 0 {
            [(4, 6), (4, 2)]
        } else {
            [(60, 62), (60, 58)]
        };
        for &mv in &castle_targets {
            if mv.0 == from {
                self.try_push_king_move(from, mv.1, moves);
            }
        }
    }

    fn try_push_king_move(&self, from: u8, to: u8, moves: &mut Vec<Move>) {
        let (flag, _) = self.validate_king_move(from, to);
        let Some(flag) = flag else { return };

        let captured_idx = self.captured_piece_idx_at(to);
        let piece_byte = (captured_idx << 4) | (PieceKind::King as u8);
        let move_mask = (from as u16) | ((to as u16) << 6) | flag.bits();
        moves.push(Move::new(move_mask, piece_byte));
    }

    /// Builds the packed [`Move`] for a pseudo-legal, already-validated
    /// destination, expanding pawn promotions into all four variants and
    /// tagging double pushes / en passant along the way.
    fn push_pseudo_legal_move(&self, piece_idx: usize, from: u8, to: u8, moves: &mut Vec<Move>) {
        let piece_type = PieceKind::from_idx(piece_idx);
        let captured_idx = self.captured_piece_idx_at(to);
        let piece_byte = (captured_idx << 4) | (piece_idx as u8);

        if piece_type == PieceKind::Pawn {
            let promotes = if self.player_turn == 0 {
                to >= 56
            } else {
                to <= 7
            };
            if promotes {
                for flag in [
                    MoveFlag::PromoQueen,
                    MoveFlag::PromoRook,
                    MoveFlag::PromoBishop,
                    MoveFlag::PromoKnight,
                ] {
                    let move_mask = (from as u16) | ((to as u16) << 6) | flag.bits();
                    moves.push(Move::new(move_mask, piece_byte));
                }
                return;
            }

            if to == self.en_passant_square {
                let move_mask = (from as u16) | ((to as u16) << 6) | MoveFlag::EpCapture.bits();
                // captured pawn isn't sitting on `to`, so no capture nibble here —
                // do_ep_capture derives the victim square independently.
                moves.push(Move::new(move_mask, (6u8 << 4) | (piece_idx as u8)));
                return;
            }

            if from.abs_diff(to) == 16 {
                let move_mask = (from as u16) | ((to as u16) << 6) | MoveFlag::DoublePush.bits();
                moves.push(Move::new(move_mask, piece_byte));
                return;
            }
        }

        let flag_bits = if captured_idx <= 5 {
            MoveFlag::Capture.bits()
        } else {
            MoveFlag::Quiet.bits()
        };
        let move_mask = (from as u16) | ((to as u16) << 6) | flag_bits;
        moves.push(Move::new(move_mask, piece_byte));
    }

    fn captured_piece_idx_at(&self, sq: u8) -> u8 {
        let enemy = (self.player_turn ^ 1) as usize;
        if (self.color_occupency[enemy] & (1u64 << sq)) == 0 {
            return 6; // none
        }
        self.get_piece_index(1u64 << sq, self.player_turn ^ 1) as u8
    }
}
