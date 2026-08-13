use crate::engine::{
    history::Undo,
    movegen::{Move, MoveFlag},
    types::PieceKind,
};

impl super::Board {
    /// Applies a validated move to the board.
    ///
    /// Creates an [`Undo`] record before modifying the position, then handles
    /// special moves such as castling, captures, pawn double pushes, and
    /// en passant target-square updates.
    ///
    /// The board occupancy caches and enemy attack mask are recalculated after
    /// the piece bitboards are modified.
    ///
    /// # Arguments
    ///
    /// * `move_info` - The packed [`Move`] describing the move to perform.
    pub fn make_move(&mut self, move_info: Move) {
        let undo = Undo::new(
            move_info,
            self.castling_rights,
            self.en_passant_square,
            self.halfmove_clock,
        );

        self.history.push(undo);

        if move_info.flags() == MoveFlag::EpCapture.bits() {
            self.do_ep_capture(move_info);
            return;
        }

        if (move_info.flags() == MoveFlag::KingCastle.bits())
            || (move_info.flags() == MoveFlag::QueenCastle.bits())
        {
            self.do_castle(move_info);
            return;
        }

        if (move_info.flags() == MoveFlag::PromoRook.bits())
            || (move_info.flags() == MoveFlag::PromoQueen.bits())
            || (move_info.flags() == MoveFlag::PromoKnight.bits())
            || (move_info.flags() == MoveFlag::PromoBishop.bits())
        {
            self.do_promotion(move_info);
            return;
        }

        self.castling_rights &= self.get_castling_rights(move_info);
        if (move_info.piece() == 0) && (move_info.flags() == MoveFlag::DoublePush.bits()) {
            self.en_passant_square = if self.player_turn == 0 {
                move_info.from() + 8
            } else {
                move_info.from() - 8
            }
        } else {
            self.en_passant_square = 64;
        }

        if (move_info.captured_piece() > 5) || (move_info.piece() == 0) {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        };

        if move_info.captured_piece() <= 5 {
            let captured_board =
                self.pieces[(self.player_turn ^ 1) as usize][move_info.captured_piece() as usize];

            self.pieces[(self.player_turn ^ 1) as usize][move_info.captured_piece() as usize] =
                captured_board ^ (1u64 << move_info.to());
        };

        let piece_board = self.pieces[self.player_turn as usize][move_info.piece() as usize];

        self.pieces[self.player_turn as usize][move_info.piece() as usize] =
            piece_board ^ ((1u64 << move_info.from()) | 1u64 << move_info.to());

        self.player_turn ^= 1;
        self.fullmove_clock += 1;

        self.init();
    }

    fn do_ep_capture(&mut self, mv: Move) {
        let capturing_idx = if (self.player_turn ^ 1) == 0 {
            mv.to() + 8
        } else {
            mv.to() - 8
        };

        self.pieces[(self.player_turn ^ 1) as usize][PieceKind::Pawn as usize] ^=
            1u64 << capturing_idx;

        self.pieces[self.player_turn as usize][PieceKind::Pawn as usize] ^=
            (1u64 << mv.from()) | (1u64 << mv.to());

        self.player_turn ^= 1;
        self.fullmove_clock += 1;
        self.en_passant_square = 64;

        self.init();
    }

    /// Executes the board changes required for a castling move.
    ///
    /// This function is responsible for moving both the king and the rook
    /// when a castling move is performed.
    ///
    /// # Arguments
    ///
    /// * `mv` - The [`Move`] containing the castling move information.
    fn do_castle(&mut self, mv: Move) {
        let color = self.player_turn as usize;
        let rook_board = self.pieces[color][PieceKind::Rook as usize];

        // Move the king from its original square to its castling square.
        self.pieces[color][PieceKind::King as usize] ^= (1u64 << mv.from()) | (1u64 << mv.to());

        match MoveFlag::from_bits(mv.flags()) {
            MoveFlag::KingCastle => {
                if color == 0 {
                    // White: h1 -> f1
                    self.pieces[color][PieceKind::Rook as usize] =
                        rook_board ^ ((1u64 << 7) | (1u64 << 5));

                    // Remove White kingside castling right (K = 0x08).
                    self.castling_rights &= 0x03;
                } else {
                    // Black: h8 -> f8
                    self.pieces[color][PieceKind::Rook as usize] =
                        rook_board ^ ((1u64 << 63) | (1u64 << 61));

                    // Remove Black kingside castling right (k = 0x02).
                    self.castling_rights &= 0x0C;
                }
            }

            MoveFlag::QueenCastle => {
                if color == 0 {
                    // White: a1 -> d1
                    self.pieces[color][PieceKind::Rook as usize] =
                        rook_board ^ ((1u64 << 0) | (1u64 << 3));

                    // Remove White queenside castling right (Q = 0x04).
                    self.castling_rights &= 0x03;
                } else {
                    // Black: a8 -> d8
                    self.pieces[color][PieceKind::Rook as usize] =
                        rook_board ^ ((1u64 << 56) | (1u64 << 59));

                    // Remove Black queenside castling right (q = 0x01).
                    self.castling_rights &= 0x0C;
                }
            }

            _ => {}
        };

        self.player_turn ^= 1;
        self.fullmove_clock += 1;

        self.init();
    }

    /// Reverts the most recently recorded move.
    ///
    /// Restores the game-state information stored in the most recent
    /// [`Undo`] record, including:
    /// - Castling rights.
    /// - En passant square.
    /// - Halfmove clock.
    ///
    /// # Panics
    ///
    /// Panics if the history stack is empty because `unwrap()` is used.
    ///
    /// # Note
    ///
    /// This currently restores only the stored metadata. The piece bitboards,
    /// player turn, and other position data are not restored here yet.
    pub fn undo_move(&mut self) {
        let undo = self.history.pop().unwrap();
        self.castling_rights = undo.castling_rights;
        self.en_passant_square = undo.en_passant_square;
        self.halfmove_clock = undo.halfmove_clock;
        self.zobrist_hash = undo.zobrist_hash;

        let mv = undo.mv;

        if mv.flags() == MoveFlag::EpCapture.bits() {
            self.undo_ep_capture(mv);
            return;
        }

        if (mv.flags() == MoveFlag::KingCastle.bits())
            || (mv.flags() == MoveFlag::QueenCastle.bits())
        {
            self.undo_castle(mv);
            return;
        }

        if (mv.flags() == MoveFlag::PromoQueen.bits())
            || (mv.flags() == MoveFlag::PromoRook.bits())
            || (mv.flags() == MoveFlag::PromoBishop.bits())
            || (mv.flags() == MoveFlag::PromoKnight.bits())
        {
            self.undo_promotion(mv);
            return;
        }

        self.player_turn ^= 1;

        if mv.captured_piece() <= 5 {
            let captured_board =
                self.pieces[(self.player_turn ^ 1) as usize][mv.captured_piece() as usize];

            self.pieces[(self.player_turn ^ 1) as usize][mv.captured_piece() as usize] =
                captured_board ^ (1u64 << mv.to());
        }

        self.pieces[self.player_turn as usize][mv.piece() as usize] = self.pieces
            [self.player_turn as usize][mv.piece() as usize]
            ^ ((1u64 << mv.from()) | 1u64 << mv.to());

        self.fullmove_clock -= 1;

        self.init();
    }

    /// Returns the castling rights for the current position.
    ///
    /// # Returns
    ///
    /// A `u8` bitmask representing the available castling rights.
    ///
    /// # Note
    ///
    /// The current implementation always returns `0x0F`.
    fn get_castling_rights(&self, mv: Move) -> u8 {
        let piece = PieceKind::from_idx(mv.piece() as usize);

        match piece {
            PieceKind::King => {
                if self.player_turn == 0 {
                    // White king moved: remove K and Q.
                    0x03
                } else {
                    // Black king moved: remove k and q.
                    0x0C
                }
            }

            PieceKind::Rook => {
                match self.player_turn {
                    0 => match mv.from() {
                        0 => 0x0B, // White rook a1 -> remove Q
                        7 => 0x07, // White rook h1 -> remove K
                        _ => 0x0F, // Other white rook
                    },

                    1 => match mv.from() {
                        56 => 0x0E, // Black rook a8 -> remove q
                        63 => 0x0D, // Black rook h8 -> remove k
                        _ => 0x0F,  // Other black rook
                    },

                    _ => 0x0F,
                }
            }

            _ => 0x0F,
        }
    }

    fn do_promotion(&mut self, mv: Move) {
        self.pieces[self.player_turn as usize][PieceKind::Pawn as usize] ^= 1u64 << mv.from();

        let promoted = match MoveFlag::from_bits(mv.flags()) {
            MoveFlag::PromoQueen => PieceKind::Queen,
            MoveFlag::PromoRook => PieceKind::Rook,
            MoveFlag::PromoBishop => PieceKind::Bishop,
            MoveFlag::PromoKnight => PieceKind::Knight,
            _ => PieceKind::Pawn,
        };

        self.pieces[self.player_turn as usize][promoted as usize] ^= 1u64 << mv.to();

        // A promotion can also be a capture (e.g. pawn takes rook while
        // promoting) — clear the captured piece, same as the normal-move
        // path does. Without this, both the new piece and the captured one
        // occupy the destination square simultaneously.
        if mv.captured_piece() <= 5 {
            let captured_board =
                self.pieces[(self.player_turn ^ 1) as usize][mv.captured_piece() as usize];
            self.pieces[(self.player_turn ^ 1) as usize][mv.captured_piece() as usize] =
                captured_board ^ (1u64 << mv.to());
        }

        self.castling_rights &= self.get_castling_rights(mv);
        self.en_passant_square = 64;
        self.halfmove_clock = 0; // pawn move, always resets

        self.player_turn ^= 1;
        self.fullmove_clock += 1;

        self.init();
    }

    fn undo_ep_capture(&mut self, mv: Move) {
        self.player_turn ^= 1; // back to the side that made the move

        self.pieces[self.player_turn as usize][PieceKind::Pawn as usize] ^=
            (1u64 << mv.from()) | (1u64 << mv.to());

        let captured_idx = if (self.player_turn ^ 1) == 0 {
            mv.to() + 8
        } else {
            mv.to() - 8
        };
        self.pieces[(self.player_turn ^ 1) as usize][PieceKind::Pawn as usize] ^=
            1u64 << captured_idx;

        self.fullmove_clock -= 1;
        self.init();
    }

    fn undo_castle(&mut self, mv: Move) {
        self.player_turn ^= 1; // back to the side that castled
        let color = self.player_turn as usize;

        self.pieces[color][PieceKind::King as usize] ^= (1u64 << mv.from()) | (1u64 << mv.to());

        match MoveFlag::from_bits(mv.flags()) {
            MoveFlag::KingCastle => {
                if color == 0 {
                    self.pieces[color][PieceKind::Rook as usize] ^= (1u64 << 7) | (1u64 << 5);
                } else {
                    self.pieces[color][PieceKind::Rook as usize] ^= (1u64 << 63) | (1u64 << 61);
                }
            }
            MoveFlag::QueenCastle => {
                if color == 0 {
                    self.pieces[color][PieceKind::Rook as usize] ^= (1u64 << 0) | (1u64 << 3);
                } else {
                    self.pieces[color][PieceKind::Rook as usize] ^= (1u64 << 56) | (1u64 << 59);
                }
            }
            _ => {}
        }

        self.fullmove_clock -= 1;
        self.init();
    }

    fn undo_promotion(&mut self, mv: Move) {
        self.player_turn ^= 1; // back to the side that promoted
        let color = self.player_turn as usize;

        let promoted = match MoveFlag::from_bits(mv.flags()) {
            MoveFlag::PromoQueen => PieceKind::Queen,
            MoveFlag::PromoRook => PieceKind::Rook,
            MoveFlag::PromoBishop => PieceKind::Bishop,
            MoveFlag::PromoKnight => PieceKind::Knight,
            _ => PieceKind::Pawn,
        };

        self.pieces[color][promoted as usize] ^= 1u64 << mv.to();
        self.pieces[color][PieceKind::Pawn as usize] ^= 1u64 << mv.from();

        if mv.captured_piece() <= 5 {
            let captured_board = self.pieces[color ^ 1][mv.captured_piece() as usize];
            self.pieces[color ^ 1][mv.captured_piece() as usize] =
                captured_board ^ (1u64 << mv.to());
        }

        self.fullmove_clock -= 1;
        self.init();
    }
}
