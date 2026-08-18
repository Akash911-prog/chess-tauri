pub mod check;
pub mod from_fen;
pub mod legal_moves;
pub mod moves;
pub mod validation;

use crate::{
    dto::{
        CommandError, GameState, Legality, MoveInfo, MoveType, PieceInfo, PieceKindDto,
        PromotionPiece, Response, SquareChange,
    },
    engine::{
        bitboard::BitBoard,
        constants::INITIAL_BOARD,
        history::HistoryManager,
        movegen::{
            Move,
            MoveFlag::{self, PromoBishop, PromoKnight, PromoQueen, PromoRook},
            MoveGen,
        },
        types::{Color, PieceKind},
    },
};

#[derive(Debug, Clone)]
pub struct Board {
    // position info
    pub pieces: [[BitBoard; 6]; 2],
    pub color_occupency: [BitBoard; 2],
    pub total_occupency: BitBoard,
    pub kings: [BitBoard; 2],

    pub player_turn: u8,       // 0 = white, 1 = black
    pub castling_rights: u8,   //KQkq
    pub promotion: u8,         // 4 = Q, 3 = R, 2 = B, 1 = N
    pub en_passant_square: u8, // index of sq
    pub halfmove_clock: u8,    // clock
    pub fullmove_clock: u16,

    pub enemy_attack_mask: BitBoard,
    pub pinned_pieces: [Option<BitBoard>; 64],

    pub zobrist_hash: u64, // TODO: Add the actualy hash logic and table logic later. Version 2.0

    pub move_gen: MoveGen,
    pub history: HistoryManager,
}

impl Board {
    /// Creates a new chess board in the initial starting position.
    ///
    /// The board starts with:
    /// - White to move.
    /// - All four castling rights enabled.
    /// - No en passant target square.
    /// - Move clocks reset.
    /// - Empty occupancy caches, which are populated by [`Board::init`].
    ///
    /// # Returns
    ///
    /// A new [`Board`] containing the initial chess position.
    pub fn new() -> Board {
        Board {
            pieces: INITIAL_BOARD,
            color_occupency: [BitBoard::EMPTY, BitBoard::EMPTY],
            total_occupency: BitBoard::EMPTY,
            kings: [BitBoard::EMPTY, BitBoard::EMPTY],

            player_turn: 0,
            castling_rights: 0x0F,
            promotion: 0,
            en_passant_square: 64,
            halfmove_clock: 0,
            fullmove_clock: 0,

            zobrist_hash: 0,

            enemy_attack_mask: BitBoard::EMPTY,
            pinned_pieces: [None; 64],

            move_gen: MoveGen::new(),
            history: HistoryManager::new(),
        }
    }

    pub fn reset(&mut self) {
        self.pieces = INITIAL_BOARD;
        self.color_occupency = [BitBoard::EMPTY, BitBoard::EMPTY];
        self.total_occupency = BitBoard::EMPTY;
        self.kings = [BitBoard::EMPTY, BitBoard::EMPTY];
        self.player_turn = 0;
        self.castling_rights = 0x0F;
        self.promotion = 0;
        self.en_passant_square = 64;
        self.halfmove_clock = 0;
        self.fullmove_clock = 0;
        self.zobrist_hash = 0;
    }

    /// Calculates the occupancy bitboard for a given color.
    ///
    /// Combines all six piece-type bitboards belonging to `color`
    /// using a bitwise OR.
    ///
    /// # Arguments
    ///
    /// * `color` - The color whose occupied squares should be calculated.
    ///
    /// # Returns
    ///
    /// A [`BitBoard`] containing every square occupied by the given color.
    // fn get_color_occupency(&self, color: Color) -> BitBoard {
    //     self.pieces[color as usize]
    //         .iter()
    //         .fold(BitBoard::EMPTY, |a, b| a | *b)
    // }

    // /// Returns the squares currently occupied by White pieces.
    // ///
    // /// # Returns
    // ///
    // /// A [`BitBoard`] containing all White pieces.
    // pub fn get_white_occupency(&self) -> BitBoard {
    //     self.color_occupency[Color::White]
    // }

    // /// Returns the occupancy bitboard for both sides combined.
    // ///
    // /// # Returns
    // ///
    // /// A [`BitBoard`] containing every occupied square on the board.
    // pub fn get_black_occupency(&self) -> BitBoard {
    //     self.color_occupency[Color::Black]
    // }

    // pub fn get_current_board(&self) -> BitBoard {
    //     self.total_occupency
    // }

    // pub fn friendly_pieces(&self) -> BitBoard {
    //     self.color_occupency[self.player_turn as usize]
    // }

    /// Recalculates all cached occupancy information and the enemy attack mask.
    ///
    /// This should be called after modifying the underlying piece bitboards.
    ///
    /// Updates:
    /// - White occupancy.
    /// - Black occupancy.
    /// - Total board occupancy.
    /// - Enemy attack mask.
    pub fn init(&mut self) {
        self.update_state();

        self.update_enemy_attack_mask(self.player_turn ^ 1);
        self.pinned_pieces = self.compute_pinned_pieces();

        self.kings = [
            self.pieces[0][PieceKind::King as usize],
            self.pieces[1][PieceKind::King as usize],
        ]
    }

    pub fn update_state(&mut self) {
        self.color_occupency[0] = self.pieces[0].iter().fold(BitBoard::EMPTY, |a, b| a | *b);
        self.color_occupency[1] = self.pieces[1].iter().fold(BitBoard::EMPTY, |a, b| a | *b);
        self.total_occupency = self.color_occupency[0] | self.color_occupency[1];
    }

    /// Parses and range-checks both squares from the frontend's algebraic notation.
    fn parse_move_squares(&self, move_info: &MoveInfo) -> Result<(u8, u8), CommandError> {
        let from = self.notation_to_index(&move_info.from);
        let to = self.notation_to_index(&move_info.to);

        if from > 63 {
            return Err(CommandError::InvalidSquareIndex {
                square: from as u16,
            });
        }
        if to > 63 {
            return Err(CommandError::InvalidSquareIndex { square: to as u16 });
        }
        Ok((from, to))
    }

    /// Sets promotion state/flags if requested, and returns the promoted piece's
    /// DTO kind (for rendering) alongside the flag bits to OR into the move mask.
    fn apply_promotion_flags(
        &mut self,
        from: u8,
        to: u8,
        promotion: Option<PromotionPiece>,
    ) -> (u16, Option<PieceKindDto>) {
        let mut move_mask = ((to as u16) << 6) | (from as u16);

        let Some(promo) = promotion else {
            return (move_mask, None);
        };

        let (bits, promotion_value, kind) = match promo {
            PromotionPiece::Queen => (PromoQueen.bits(), 4, PieceKindDto::Queen),
            PromotionPiece::Rook => (PromoRook.bits(), 3, PieceKindDto::Rook),
            PromotionPiece::Bishop => (PromoBishop.bits(), 2, PieceKindDto::Bishop),
            PromotionPiece::Knight => (PromoKnight.bits(), 1, PieceKindDto::Knight),
        };

        self.promotion = promotion_value; // NOTE: still `=` here, not `|=` — see callout below
        move_mask |= bits;
        (move_mask, Some(kind))
    }

    /// Validates a king move and, if it's a castle, builds the rook's square changes.
    /// Returns `None` if the move is illegal.
    fn resolve_king_move(&self, from: u8, to: u8) -> Option<(MoveFlag, Option<Vec<SquareChange>>)> {
        let (flag, (rook_from, rook_to)) = self.validate_king_move(from, to);
        let flag = flag?;

        if flag == MoveFlag::KingCastle || flag == MoveFlag::QueenCastle {
            let piece_info = PieceInfo::new(PieceKindDto::Rook, Color::from(self.player_turn));
            let changes = vec![
                SquareChange::new(self.index_to_notation(rook_to), Some(piece_info)),
                SquareChange::new(self.index_to_notation(rook_from), None),
            ];
            return Some((flag, Some(changes)));
        }

        Some((flag, None))
    }

    /// Builds the {from: empty, to: piece} change pair for a normal/promoting move.
    /// Uses `promoted_kind` in place of the original piece type when present —
    /// this is what makes promotions actually render correctly.
    fn build_move_changes(
        &self,
        from: u8,
        to: u8,
        piece_type: PieceKind,
        promoted_kind: Option<PieceKindDto>,
    ) -> Vec<SquareChange> {
        let kind = promoted_kind.unwrap_or_else(|| piece_type.into());
        vec![
            SquareChange::new(self.index_to_notation(from), None),
            SquareChange::new(
                self.index_to_notation(to),
                Some(PieceInfo::new(kind, Color::from(self.player_turn))),
            ),
        ]
    }

    /// Builds the extra "captured pawn's square goes empty" change for en passant.
    fn build_ep_capture_change(&self, from: u8, to: u8) -> SquareChange {
        let captured_idx = if to > from { to - 8 } else { to + 8 };
        SquareChange::new(self.index_to_notation(captured_idx), None)
    }

    /// Determines capture status against `to` and packs the final `Move`.
    /// Returns `None` if the destination is occupied but the piece there
    /// can't be identified (shouldn't happen on a consistent board — treat as illegal).
    fn build_packed_move(&self, move_mask: u16, to: u8, piece_idx: usize) -> Option<Move> {
        let mut move_mask = move_mask;
        let side = self.player_turn;
        let captured_piece_mask = 1u64 << to;

        if (self.color_occupency[(side ^ 1) as usize] & captured_piece_mask) == 0 {
            let piece = (6u8 << 4) | (piece_idx as u8);
            return Some(Move::new(move_mask, piece));
        }

        let captured_piece_idx = self.get_piece_index(captured_piece_mask, side ^ 1);
        if captured_piece_idx > 5 {
            return None;
        }
        let captured_piece = ((captured_piece_idx as u8) << 4) | (piece_idx as u8);
        move_mask |= MoveFlag::Capture.bits();
        Some(Move::new(move_mask, captured_piece))
    }

    pub fn parse_react_move(&mut self, move_info: MoveInfo) -> Result<Legality, CommandError> {
        if self.get_game_state() != GameState::InProgress {
            return Err(CommandError::GameAlreadyOver);
        }

        let (from, to) = self.parse_move_squares(&move_info)?;

        let piece_idx = self.get_piece_index(1u64 << from, self.player_turn);
        if piece_idx > 5 {
            return Err(CommandError::EmptySquare { square: from });
        }
        let piece_type = PieceKind::from_idx(piece_idx);

        let (mut move_mask, promoted_kind) =
            self.apply_promotion_flags(from, to, move_info.promotion);

        let mut square_changes = Vec::new();
        let mut move_type = MoveType::Normal;

        if piece_type == PieceKind::King {
            let Some((flag, castle_changes)) = self.resolve_king_move(from, to) else {
                return Ok(Legality::Illegal);
            };
            move_mask |= flag.bits();
            if let Some(changes) = castle_changes {
                square_changes.extend(changes);
                move_type = MoveType::Castling;
            }
        } else if !self.validate_move(piece_type, from, to, None) {
            return Ok(Legality::Illegal);
        }

        square_changes.extend(self.build_move_changes(from, to, piece_type, promoted_kind));

        if piece_type == PieceKind::Pawn {
            if from.abs_diff(to) == 16 {
                move_mask |= MoveFlag::DoublePush.bits();
            }
            if self.en_passant_square == to {
                square_changes.push(self.build_ep_capture_change(from, to));
                move_type = MoveType::EnPassant;
                move_mask |= MoveFlag::EpCapture.bits();
            }
        }

        let Some(new_move) = self.build_packed_move(move_mask, to, piece_idx) else {
            return Ok(Legality::Illegal);
        };

        self.make_move(new_move);
        Ok(Legality::Legal(
            self.build_response(square_changes, move_type),
        ))
    }

    fn build_response(&self, square_changes: Vec<SquareChange>, move_type: MoveType) -> Response {
        let game_state = self.get_game_state();
        let mut winner = None;

        if game_state == GameState::Checkmate {
            winner = Some(Color::from(self.player_turn ^ 1));
        }

        let response = Response {
            changes: square_changes,
            condition: game_state,
            winner: winner,
            move_type,
        };

        response
    }

    pub fn index_to_notation(&self, index: u8) -> String {
        if index > 63 {
            return "".to_string();
        }
        const FILES: [&str; 8] = ["a", "b", "c", "d", "e", "f", "g", "h"];
        let file = FILES[index as usize % 8];
        let rank = (index as usize / 8) + 1;
        format!("{}{}", file, rank)
    }

    /// Recalculates the attack mask for the opponent of the current player.
    ///
    /// Generates all pseudo/legal moves available to the opponent and combines
    /// their destination bitboards into a single attack mask.
    ///
    /// The resulting mask is stored in `enemy_attack_mask` and is used,
    /// among other things, when validating king moves and castling.
    fn update_enemy_attack_mask(&mut self, color: u8) {
        let enemy_possible_moves = self.get_all_legal_moves(color, true);
        let mask: BitBoard = enemy_possible_moves
            .iter()
            .fold(BitBoard::EMPTY, |acc, &x| acc | x);

        self.enemy_attack_mask = mask;
    }

    // board/check.rs, or a new board/pins.rs — your call
    pub fn compute_pinned_pieces(&self) -> [Option<BitBoard>; 64] {
        let mut pinned: [Option<BitBoard>; 64] = [None; 64];

        let player = self.player_turn as usize;
        let enemy = (self.player_turn ^ 1) as usize;
        let king_board = self.pieces[player][PieceKind::King as usize];
        let king_idx = king_board.lsb() as usize;
        let queen_board = self.pieces[enemy][PieceKind::Queen as usize];

        let occ_without_friendly = self.total_occupency & !self.color_occupency[player];

        let rook_pinners = BitBoard(self.move_gen.gen_rook_attacks(
            king_idx,
            occ_without_friendly,
            self.player_turn,
            false,
            &self.kings,
        )) & (self.pieces[enemy][PieceKind::Rook as usize] | queen_board);
        let bishop_pinners = BitBoard(self.move_gen.gen_bishop_attacks(
            king_idx,
            occ_without_friendly,
            self.player_turn,
            false,
            &self.kings,
        )) & (self.pieces[enemy][PieceKind::Bishop as usize] | queen_board);

        for mut pinners in [rook_pinners, bishop_pinners] {
            while let Some(pinner_sq) = pinners.pop_lsb() {
                let between = self.move_gen.ray_between(king_idx, pinner_sq as usize);
                let blockers = between & self.color_occupency[player];

                if blockers.count() == 1 {
                    let pinned_sq = blockers.lsb() as usize;
                    pinned[pinned_sq] = Some(between | BitBoard(1u64 << pinner_sq));
                }
            }
        }

        pinned
    }

    /// Converts algebraic chess notation into a zero-based bitboard index.
    ///
    /// The expected format is a two-character square such as `"a1"` or `"h8"`.
    ///
    /// # Arguments
    ///
    /// * `notation` - Chess square notation consisting of a file (`a`-`h`)
    ///   followed by a rank (`1`-`8`).
    ///
    /// # Returns
    ///
    /// A square index from `0` to `63`.
    ///
    /// Returns `64` if the file or rank is outside the valid chess board.
    pub fn notation_to_index(&self, notation: &str) -> u8 {
        let file = notation.chars().nth(0).unwrap() as u8 - 'a' as u8;
        let rank = notation.chars().nth(1).unwrap() as u8 - '1' as u8;

        if (file > 7) || (rank > 7) {
            return 64; // index beyond 63 means illegal move. That is what is returned
        }
        rank * 8 + file
    }

    /// Finds which piece type occupies a given square for a side.
    ///
    /// Searches the six piece-type bitboards belonging to `side` and returns
    /// the index of the first bitboard containing the requested square.
    ///
    /// # Arguments
    ///
    /// * `mask` - A bitboard mask identifying the square to inspect.
    /// * `side` - The side whose pieces should be searched.
    ///
    /// # Returns
    ///
    /// A piece index from `0` to `5`:
    ///
    /// * `0` - Pawn
    /// * `1` - Knight
    /// * `2` - Bishop
    /// * `3` - Rook
    /// * `4` - Queen
    /// * `5` - King
    ///
    /// Returns `6` when no piece occupies the requested square.
    fn get_piece_index(&self, mask: u64, side: u8) -> usize {
        let piece_idx_option = self.pieces[side as usize]
            .iter()
            .position(|&piece_bitboard| (piece_bitboard & mask) != BitBoard(0));

        let piece_idx = match piece_idx_option {
            Some(idx) => idx,
            None => return 6, // Out of bound index. Meaning no pieces found
        };

        piece_idx
    }

    /// Determines whether the current player has no legal moves remaining.
    ///
    /// This is used to determine whether the game has reached a terminal
    /// position such as checkmate or stalemate.
    ///
    /// # Returns
    ///
    /// `true` if the current player has zero generated moves, otherwise `false`.
    ///
    /// # Note
    ///
    /// This function only checks whether moves exist. It does not distinguish
    /// between checkmate and stalemate.
    fn get_game_state(&self) -> GameState {
        let check_info = self.check_for_check();
        let has_legal_moves = self.any_legal_move_exists();

        if !has_legal_moves && check_info.is_check {
            GameState::Checkmate
        } else if !has_legal_moves {
            GameState::Stalemate
        } else {
            GameState::InProgress
        }
    }

    fn any_legal_move_exists(&self) -> bool {
        let color = self.player_turn;
        let friendly = self.color_occupency[color as usize];
        let enemy = self.color_occupency[(color ^ 1) as usize];
        let occupied = self.total_occupency;

        let check_info = self.check_for_check();

        for piece_idx in 0..6 {
            let piece_type = PieceKind::from_idx(piece_idx);
            let mut piece_board = self.pieces[color as usize][piece_idx];

            while let Some(from) = piece_board.pop_lsb() {
                let pseudo_moves = self.move_gen.get_legal_moves_by_piece(
                    piece_type,
                    occupied,
                    from as usize,
                    color,
                    enemy,
                    friendly,
                    false,
                    self.en_passant_square,
                    &self.kings,
                );

                let Some(mut destinations) = pseudo_moves else {
                    continue;
                };

                while let Some(to) = destinations.pop_lsb() {
                    if piece_type == PieceKind::King {
                        if self.validate_king_move(from, to).0.is_some() {
                            return true;
                        }
                    } else if self.validate_move(piece_type, from, to, Some(check_info)) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Generates move bitboards for every piece belonging to a given player.
    ///
    /// Each entry in the returned vector represents the legal destinations
    /// generated for one piece on the board.
    ///
    /// # Arguments
    ///
    /// * `player_turn` - The color whose moves should be generated:
    ///   `0` for White and `1` for Black.
    ///
    /// # Returns
    ///
    /// A vector of [`BitBoard`] values containing the generated destinations
    /// for every piece belonging to `player_turn`.
    pub fn get_all_legal_moves(&self, player_turn: u8, attack_only: bool) -> Vec<BitBoard> {
        let color = player_turn;
        let friendly = self.color_occupency[color as usize];
        let enemy = self.color_occupency[(color ^ 1) as usize];
        let occupied = self.total_occupency;

        let mut all_moves = Vec::with_capacity(16);
        for piece_idx in 0..6 {
            let piece_type = PieceKind::from_idx(piece_idx);
            let mut bb = self.pieces[color as usize][piece_idx];
            while bb.0 != 0 {
                let sq = match bb.pop_lsb() {
                    Some(sq) => sq,
                    None => break,
                };
                if let Some(moves) = self.move_gen.get_legal_moves_by_piece(
                    piece_type,
                    occupied,
                    sq.into(),
                    color,
                    enemy,
                    friendly,
                    attack_only,
                    self.en_passant_square,
                    &self.kings,
                ) {
                    all_moves.push(moves);
                }
            }
        }
        all_moves
    }

    pub fn unoccupied_or_safe(&self) -> BitBoard {
        !self.enemy_attack_mask & !self.total_occupency
    }
}
