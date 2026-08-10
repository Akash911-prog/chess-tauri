pub mod check;
pub mod from_fen;
pub mod moves;
pub mod validation;

use crate::{
    dto::{CommandError, GameState, Legality, MoveInfo, PromotionPiece, Response},
    engine::{
        bitboard::BitBoard,
        constants::INITIAL_BOARD,
        history::HistoryManager,
        movegen::{
            Move,
            MoveFlag::{self, PromoBishop, PromoKnight, PromoQueen, PromoRook},
            MoveGen,
        },
        types::PieceKind,
    },
};

pub struct Board {
    // position info
    pub pieces: [[BitBoard; 6]; 2],
    pub color_occupency: [BitBoard; 2],
    pub total_occupency: BitBoard,

    pub player_turn: u8,       // 0 = white, 1 = black
    pub castling_rights: u8,   //KQkq
    pub promotion: u8,         // 0 = Q, 1 = R, 2 = B, 3 = N
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
    }

    pub fn update_state(&mut self) {
        self.color_occupency[0] = self.pieces[0].iter().fold(BitBoard::EMPTY, |a, b| a | *b);
        self.color_occupency[1] = self.pieces[1].iter().fold(BitBoard::EMPTY, |a, b| a | *b);
        self.total_occupency = self.color_occupency[0] | self.color_occupency[1];
    }

    /// Parses and validates a move received from the React frontend.
    ///
    /// Converts algebraic square notation such as `"e2"` and `"e4"` into
    /// bitboard square indices, identifies the moving piece, determines
    /// promotion information, validates the move, constructs the packed
    /// [`Move`], and finally applies it to the board.
    ///
    /// # Arguments
    ///
    /// * `move_info` - The frontend [`MoveInfo`] containing source, destination,
    ///   and optional promotion information.
    ///
    /// # Returns
    ///
    /// * `Ok(Legality::Legal)` - The move was valid and was applied.
    /// * `Ok(Legality::Illegal)` - The move was syntactically valid but illegal.
    /// * `Err(CommandError)` - The move could not be processed because of an
    ///   invalid square, empty source square, or game state error.
    pub fn parse_react_move(&mut self, move_info: MoveInfo) -> Result<Legality, CommandError> {
        if self.get_game_state() != GameState::InProgress {
            return Err(CommandError::GameAlreadyOver);
        }

        // --- Parse & validate square notation ---
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

        // 16-bit move encoding: [6-bit from][6-bit to]
        let mut move_mask = ((to as u16) << 6) | (from as u16);
        let piece_mask = 1u64 << from;
        let captured_piece_mask = 1u64 << to;
        let side = self.player_turn;

        // --- Promotion check & Flag setting ---
        let (promo_piece, is_promotion) = match move_info.promotion {
            Some(promo) => (promo, true),
            None => (PromotionPiece::B, false),
        };

        if is_promotion {
            match promo_piece {
                PromotionPiece::Q => {
                    self.promotion |= 4;
                    move_mask |= PromoQueen.bits()
                }
                PromotionPiece::R => {
                    self.promotion |= 3;
                    move_mask |= PromoRook.bits()
                }
                PromotionPiece::B => {
                    self.promotion |= 2;
                    move_mask |= PromoBishop.bits()
                }
                PromotionPiece::N => {
                    self.promotion |= 1;
                    move_mask |= PromoKnight.bits()
                }
            }
        }

        // --- Identify the moving piece ---
        // Find the piece index (0 = Pawn, 1 = Knight, etc.) that occupies the 'from' square
        let piece_idx = self.get_piece_index(piece_mask, side);
        if piece_idx > 5 {
            return Err(CommandError::EmptySquare { square: from });
        }
        println!("piece_idx: {}", piece_idx);
        let piece_type = PieceKind::from_idx(piece_idx);

        println!("from: {}, to: {}, piece_type: {:?}", from, to, piece_type);

        // --- Legality check ---
        if piece_type == PieceKind::King {
            let flag = match self.validate_king_move(from, to) {
                Some(flag) => flag,
                None => return Ok(Legality::Illegal),
            };

            move_mask |= flag.bits();
        } else {
            if !self.validate_move(piece_type, from, to) {
                println!("Invalid move");
                return Ok(Legality::Illegal);
            }
        }

        // double push check and flag set
        if (piece_type == PieceKind::Pawn) && (from.abs_diff(to) == 16) {
            move_mask |= MoveFlag::DoublePush.bits();
        }

        // En passant check and flag set
        if (piece_type == PieceKind::Pawn) && (self.en_passant_square == to) {
            move_mask |= MoveFlag::EpCapture.bits();
        }

        // --- Determine capture status & build the packed Move ---
        let new_move = if (self.color_occupency[(side ^ 1) as usize] & captured_piece_mask) == 0 {
            // Target square is empty - quiet move, no capture
            let piece = (6u8 << 4) | (piece_idx as u8);
            Move::new(move_mask, piece)
        } else {
            // Target square is occupied by an enemy piece - capture move
            let captured_piece_idx = self.get_piece_index(captured_piece_mask, side ^ 1);
            if captured_piece_idx > 5 {
                println!("captured_piece_idx: {}", captured_piece_idx);
                return Ok(Legality::Illegal);
            }
            let captured_piece = ((captured_piece_idx as u8) << 4) | (captured_piece_idx as u8);
            move_mask |= MoveFlag::Capture.bits();
            Move::new(move_mask, captured_piece)
        };
        let info = self.make_move(new_move);
        Ok(Legality::Legal(info))
    }

    fn index_to_notation(&self, index: u8) -> String {
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

        let rook_pinners = BitBoard(
            self.move_gen
                .gen_rook_attacks(king_idx, occ_without_friendly),
        ) & (self.pieces[enemy][PieceKind::Rook as usize] | queen_board);
        let bishop_pinners = BitBoard(
            self.move_gen
                .gen_bishop_attacks(king_idx, occ_without_friendly),
        ) & (self.pieces[enemy][PieceKind::Bishop as usize] | queen_board);

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
    fn notation_to_index(&self, notation: &str) -> u8 {
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
                );

                let Some(mut destinations) = pseudo_moves else {
                    continue;
                };

                while let Some(to) = destinations.pop_lsb() {
                    if piece_type == PieceKind::King {
                        if self.validate_king_move(from, to).is_some() {
                            return true;
                        }
                    } else if self.validate_move(piece_type, from, to) {
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

        let mut all_moves = Vec::new();
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
                ) {
                    all_moves.push(moves);
                }
            }
        }
        all_moves
    }
}
