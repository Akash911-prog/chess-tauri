use crate::engine::{
    bitboard::BitBoard,
    constants::{B_KING_CASTLE, B_QUEEN_CASTLE, W_KING_CASTLE, W_QUEEN_CASTLE},
    movegen::MoveFlag,
    types::{CastlingRights, PieceKind},
};

impl super::Board {
    /// Checks whether a non-king piece can legally move from one square to another.
    ///
    /// Uses the move generator to obtain the piece's possible destinations and
    /// checks whether `to` is included in that set.
    ///
    /// # Arguments
    ///
    /// * `piece_type` - Type of piece being moved.
    /// * `from` - Source square index.
    /// * `to` - Destination square index.
    ///
    /// # Returns
    ///
    /// `true` if the destination is present in the generated move set,
    /// otherwise `false`.
    pub fn validate_move(&self, piece_type: PieceKind, from: u8, to: u8) -> bool {
        if let Some(pin_line) = self.pinned_pieces[from as usize] {
            if (pin_line & BitBoard(1u64 << to)) == 0 {
                return false;
            }
        }

        if (piece_type == PieceKind::Pawn) && (to == self.en_passant_square) {
            return self.validate_ep(from, to);
        }

        let possible_moves = self.move_gen.get_legal_moves_by_piece(
            piece_type,
            self.total_occupency,
            from as usize,
            self.player_turn,
            self.color_occupency[(self.player_turn ^ 1) as usize],
            self.color_occupency[self.player_turn as usize],
            false,
        );

        let current_move = (1u64 << from) | (1u64 << to);

        if let Some(possible_moves) = possible_moves {
            if (possible_moves & current_move) == (1u64 << to) {
                let check_info = self.check_for_check();
                if check_info.is_check {
                    return self.validate_move_with_check(to, &check_info, piece_type);
                }
                return true;
            }
        }
        false
    }

    fn validate_ep(&self, from: u8, to: u8) -> bool {
        let capturing_idx = if (self.player_turn ^ 1) == 0 {
            to + 8
        } else {
            to - 8
        };

        if (self.pieces[self.player_turn as usize][PieceKind::Pawn as usize] & (1u64 << from)) != 0
        {
            if (self.pieces[(self.player_turn ^ 1) as usize][PieceKind::Pawn as usize]
                & (1u64 << capturing_idx))
                != 0
            {
                return true;
            };
        }

        return false;
    }

    /// Checks whether the king can make a normal, non-castling move.
    ///
    /// Generates the king's possible moves and removes squares currently
    /// attacked by the opponent.
    ///
    /// # Arguments
    ///
    /// * `from` - The king's current square index.
    /// * `to` - The square the king wants to move to.
    ///
    /// # Returns
    ///
    /// `true` if the destination is a legal king destination, otherwise `false`.
    fn quiet_king_push(&self, from: u8, to: u8) -> bool {
        let possible_moves = match self.move_gen.get_legal_moves_by_piece(
            PieceKind::King,
            self.total_occupency,
            from as usize,
            self.player_turn,
            self.color_occupency[(self.player_turn ^ 1) as usize],
            self.color_occupency[self.player_turn as usize],
            false,
        ) {
            Some(moves) => moves,
            None => return false,
        };

        let possible_moves = possible_moves & !self.enemy_attack_mask;
        possible_moves & (1u64 << to) != 0
    }

    /// Validates a king move, including both normal king movement and castling.
    ///
    /// Normal king movement is checked first. If that fails, the move is
    /// checked against the castling rules.
    ///
    /// # Arguments
    ///
    /// * `from` - The king's starting square index.
    /// * `to` - The king's destination square index.
    ///
    /// # Returns
    ///
    /// * `Some(MoveFlag::Quiet)` for a legal normal king move.
    /// * `Some(MoveFlag::KingCastle)` for legal kingside castling.
    /// * `Some(MoveFlag::QueenCastle)` for legal queenside castling.
    /// * `None` if the move is illegal.
    pub fn validate_king_move(&self, from: u8, to: u8) -> (Option<MoveFlag>, (u8, u8)) {
        if self.quiet_king_push(from, to) {
            return (Some(MoveFlag::Quiet), (from, to));
        };

        let castle_result = self.validate_castle(from, to);
        if let Some(castle) = castle_result.0 {
            return (Some(castle), castle_result.1);
        }

        (None, (65, 65))
    }

    /// Validates whether the king can perform a castling move.
    ///
    /// Checks the appropriate castling right, rook presence, required empty
    /// squares, and whether the king's starting, transit, and destination
    /// squares are safe from enemy attacks.
    ///
    /// # Arguments
    ///
    /// * `from` - The king's starting square index.
    /// * `to` - The king's destination square index.
    ///
    /// # Returns
    ///
    /// * `Some(MoveFlag::KingCastle)` for legal kingside castling.
    /// * `Some(MoveFlag::QueenCastle)` for legal queenside castling.
    /// * `None` if the move is not a valid castle.
    fn validate_castle(&self, from: u8, to: u8) -> (Option<MoveFlag>, (u8, u8)) {
        let move_mask = (1u64 << from) | (1u64 << to);

        let castle = match self.identify_castle_type(move_mask) {
            Some(castle) => castle,
            None => return (None, (65, 65)),
        };
        // println!("castle: {castle:?}");

        if !self.has_castling_right(castle) {
            // println!("no castling rights");
            return (None, (65, 65));
        }

        let (rook_square, empty_mask, safe_mask, flag, rook_moves) = match castle {
            CastlingRights::WhiteKingside => {
                let rook_square = 7;
                let empty_mask = (1u64 << 5) | (1u64 << 6);
                let safe_mask = (1u64 << 4) | (1u64 << 5) | (1u64 << 6);

                (
                    rook_square,
                    empty_mask,
                    safe_mask,
                    MoveFlag::KingCastle,
                    (7u8, 5u8),
                )
            }

            CastlingRights::WhiteQueenside => {
                let rook_square = 0;
                let empty_mask = (1u64 << 1) | (1u64 << 2) | (1u64 << 3);
                let safe_mask = (1u64 << 2) | (1u64 << 3) | (1u64 << 4);

                (
                    rook_square,
                    empty_mask,
                    safe_mask,
                    MoveFlag::QueenCastle,
                    (63u8, 61u8),
                )
            }

            CastlingRights::BlackKingside => {
                let rook_square = 63;
                let empty_mask = (1u64 << 61) | (1u64 << 62);
                let safe_mask = (1u64 << 60) | (1u64 << 61) | (1u64 << 62);

                (
                    rook_square,
                    empty_mask,
                    safe_mask,
                    MoveFlag::KingCastle,
                    (0u8, 3u8),
                )
            }

            CastlingRights::BlackQueenside => {
                let rook_square = 56;
                let empty_mask = (1u64 << 57) | (1u64 << 58) | (1u64 << 59);
                let safe_mask = (1u64 << 58) | (1u64 << 59) | (1u64 << 60);

                (
                    rook_square,
                    empty_mask,
                    safe_mask,
                    MoveFlag::QueenCastle,
                    (56u8, 59u8),
                )
            }
        };

        // Correct rook must actually exist.
        let rook_board = self.pieces[self.player_turn as usize][PieceKind::Rook as usize];

        if rook_board & (1u64 << rook_square) == 0 {
            // println!("rook not found");
            return (None, (65, 65));
        }

        // All required squares must be empty.
        if !self.total_occupency & empty_mask != empty_mask {
            // println!("squares not empty");
            return (None, (65, 65));
        }

        // King cannot currently be in check, cross an attacked square,
        // or land on an attacked square.
        if !self.enemy_attack_mask & safe_mask != safe_mask {
            // println!("king in check");
            return (None, (65, 65));
        }

        (Some(flag), rook_moves)
    }

    /// Identifies the type of castling represented by a king move.
    ///
    /// The function compares the combined source and destination bit mask
    /// against the four possible castling moves.
    ///
    /// # Arguments
    ///
    /// * `move_mask` - A bitmask containing exactly the king's source and
    ///   destination squares.
    ///
    /// # Returns
    ///
    /// The corresponding [`CastlingRights`] value if the mask represents
    /// castling, otherwise `None`.
    fn identify_castle_type(&self, move_mask: u64) -> Option<CastlingRights> {
        match move_mask {
            W_QUEEN_CASTLE => Some(CastlingRights::WhiteQueenside),
            W_KING_CASTLE => Some(CastlingRights::WhiteKingside),
            B_KING_CASTLE => Some(CastlingRights::BlackKingside),
            B_QUEEN_CASTLE => Some(CastlingRights::BlackQueenside),
            _ => None,
        }
    }

    /// Checks whether a specific castling right is currently available.
    ///
    /// # Arguments
    ///
    /// * `right` - The castling right to check.
    ///
    /// # Returns
    ///
    /// `true` if the requested right is present in `castling_rights`,
    /// otherwise `false`.
    fn has_castling_right(&self, right: CastlingRights) -> bool {
        self.castling_rights & right.bits() != 0
    }
}
