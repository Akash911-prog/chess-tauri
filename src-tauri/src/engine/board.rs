use crate::{
    dto::{CommandError, Legality, MoveInfo, PromotionPiece},
    engine::{
        bitboard::BitBoard,
        history::{HistoryManager, Undo},
        movegen::{
            Move,
            MoveFlag::{self, PromoBishop, PromoKnight, PromoQueen, PromoRook},
            MoveGen,
        },
        types::{CastlingRights, Color, PieceKind},
    },
};

const INITIAL_BOARD: [[BitBoard; 6]; 2] = [
    [
        BitBoard(0x0000_0000_0000_FF00),
        BitBoard(0x0000_0000_0000_0042),
        BitBoard(0x0000_0000_0000_0024),
        BitBoard(0x0000_0000_0000_0081),
        BitBoard(0x0000_0000_0000_0008),
        BitBoard(0x0000_0000_0000_0010),
    ],
    [
        BitBoard(0x00FF_0000_0000_0000),
        BitBoard(0x4200_0000_0000_0000),
        BitBoard(0x2400_0000_0000_0000),
        BitBoard(0x8100_0000_0000_0000),
        BitBoard(0x0800_0000_0000_0000),
        BitBoard(0x1000_0000_0000_0000),
    ],
];

const W_QUEEN_CASTLE: u64 = 0x0000_0000_0000_0014;
const W_KING_CASTLE: u64 = 0x0000_0000_0000_0050;

const B_QUEEN_CASTLE: u64 = 0x1400_0000_0000_0000;
const B_KING_CASTLE: u64 = 0x5000_0000_0000_0000;

const SQ_MASK_W_QUEEN_CASTLE: u64 = 0x0000_0000_0000_0078;
const SQ_MASK_W_KING_CASTLE: u64 = 0x0000_0000_0000_000E;
const SQ_MASK_B_QUEEN_CASTLE: u64 = 0x7800_0000_0000_0078;
const SQ_MASK_B_KING_CASTLE: u64 = 0x0E00_0000_0000_0078;

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
    fn get_color_occupency(&self, color: Color) -> BitBoard {
        self.pieces[color as usize]
            .iter()
            .fold(BitBoard::EMPTY, |a, b| a | *b)
    }

    /// Returns the squares currently occupied by White pieces.
    ///
    /// # Returns
    ///
    /// A [`BitBoard`] containing all White pieces.
    pub fn get_white_occupency(&self) -> BitBoard {
        self.color_occupency[Color::White]
    }

    /// Returns the occupancy bitboard for both sides combined.
    ///
    /// # Returns
    ///
    /// A [`BitBoard`] containing every occupied square on the board.
    pub fn get_black_occupency(&self) -> BitBoard {
        self.color_occupency[Color::Black]
    }

    pub fn get_current_board(&self) -> BitBoard {
        self.total_occupency
    }

    pub fn friendly_pieces(&self) -> BitBoard {
        self.color_occupency[self.player_turn as usize]
    }

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
        self.color_occupency[0] = self.pieces[0].iter().fold(BitBoard::EMPTY, |a, b| a | *b);
        self.color_occupency[1] = self.pieces[1].iter().fold(BitBoard::EMPTY, |a, b| a | *b);
        self.total_occupency = self.color_occupency[0] | self.color_occupency[1];

        self.update_enemy_attack_mask();
    }

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
    pub fn make_move(&mut self, move_info: Move) -> (bool, (u8, u8)) {
        let undo = Undo::new(
            move_info,
            self.castling_rights,
            self.en_passant_square,
            self.halfmove_clock,
        );

        if (move_info.flags() & MoveFlag::KingCastle.bits() == MoveFlag::KingCastle.bits())
            || (move_info.flags() & MoveFlag::QueenCastle.bits() == MoveFlag::QueenCastle.bits())
        {
            let (from, to) = self.do_castle(move_info);
            return (true, (from, to));
        }

        self.history.push(undo);

        self.castling_rights = self.get_castling_rights(move_info);
        if (move_info.piece() == 0)
            && ((move_info.flags() & MoveFlag::DoublePush.bits()) == MoveFlag::DoublePush.bits())
        {
            self.en_passant_square = if self.player_turn == 0 {
                move_info.from() + 8
            } else {
                move_info.from() - 8
            }
        };

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

        self.init();

        self.player_turn ^= 1;
        self.fullmove_clock += 1;
        (false, (move_info.from(), move_info.to()))
    }

    /// Executes the board changes required for a castling move.
    ///
    /// This function is responsible for moving both the king and the rook
    /// when a castling move is performed.
    ///
    /// # Arguments
    ///
    /// * `mv` - The [`Move`] containing the castling move information.
    fn do_castle(&mut self, mv: Move) -> (u8, u8) {
        let color = self.player_turn as usize;
        let rook_board = self.pieces[color][PieceKind::Rook as usize];

        // Move the king from its original square to its castling square.
        self.pieces[color][PieceKind::King as usize] ^= (1u64 << mv.from()) | (1u64 << mv.to());

        let (from, to) = match MoveFlag::from_bits(mv.flags()) {
            MoveFlag::KingCastle => {
                if color == 0 {
                    // White: h1 -> f1
                    self.pieces[color][PieceKind::Rook as usize] =
                        rook_board ^ ((1u64 << 7) | (1u64 << 5));

                    // Remove White kingside castling right (K = 0x08).
                    self.castling_rights &= 0x07;

                    (7, 5)
                } else {
                    // Black: h8 -> f8
                    self.pieces[color][PieceKind::Rook as usize] =
                        rook_board ^ ((1u64 << 63) | (1u64 << 61));

                    // Remove Black kingside castling right (k = 0x02).
                    self.castling_rights &= 0x0D;

                    (63u8, 61u8)
                }
            }

            MoveFlag::QueenCastle => {
                if color == 0 {
                    // White: a1 -> d1
                    self.pieces[color][PieceKind::Rook as usize] =
                        rook_board ^ ((1u64 << 0) | (1u64 << 3));

                    // Remove White queenside castling right (Q = 0x04).
                    self.castling_rights &= 0x0B;

                    (0, 3)
                } else {
                    // Black: a8 -> d8
                    self.pieces[color][PieceKind::Rook as usize] =
                        rook_board ^ ((1u64 << 56) | (1u64 << 59));

                    // Remove Black queenside castling right (q = 0x01).
                    self.castling_rights &= 0x0E;

                    (56, 59)
                }
            }

            _ => (65, 65),
        };

        self.init();
        self.player_turn ^= 1;
        self.fullmove_clock += 1;

        (from, to)
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
        if self.is_game_over() {
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
            Move::new(move_mask, captured_piece)
        };
        let info = self.make_move(new_move);
        Ok(Legality::Legal(
            info.0,
            (
                self.index_to_notation(info.1 .0),
                self.index_to_notation(info.1 .1),
            ),
        ))
    }

    fn index_to_notation(&self, index: u8) -> String {
        const FILES: [&str; 8] = ["a", "b", "c", "d", "e", "f", "g", "h"];
        let file = FILES[index as usize % 8];
        let rank = (index as usize / 8) + 1;
        format!("{}{}", file, rank)
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
    fn validate_move(&self, piece_type: PieceKind, from: u8, to: u8) -> bool {
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
                return true;
            }
        }
        false
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
    fn validate_king_move(&self, from: u8, to: u8) -> Option<MoveFlag> {
        if self.quiet_king_push(from, to) {
            return Some(MoveFlag::Quiet);
        };

        if let Some(flag) = self.validate_castle(from, to) {
            return Some(flag);
        };

        None
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
    fn validate_castle(&self, from: u8, to: u8) -> Option<MoveFlag> {
        let move_mask = (1u64 << from) | (1u64 << to);

        let castle = self.identify_castle_type(move_mask)?;
        println!("castle: {castle:?}");

        if !self.has_castling_right(castle) {
            println!("no castling rights");
            return None;
        }

        let (rook_square, empty_mask, safe_mask, flag) = match castle {
            CastlingRights::WhiteKingside => {
                let rook_square = 7;
                let empty_mask = (1u64 << 5) | (1u64 << 6);
                let safe_mask = (1u64 << 4) | (1u64 << 5) | (1u64 << 6);

                (rook_square, empty_mask, safe_mask, MoveFlag::KingCastle)
            }

            CastlingRights::WhiteQueenside => {
                let rook_square = 0;
                let empty_mask = (1u64 << 1) | (1u64 << 2) | (1u64 << 3);
                let safe_mask = (1u64 << 2) | (1u64 << 3) | (1u64 << 4);

                (rook_square, empty_mask, safe_mask, MoveFlag::QueenCastle)
            }

            CastlingRights::BlackKingside => {
                let rook_square = 63;
                let empty_mask = (1u64 << 61) | (1u64 << 62);
                let safe_mask = (1u64 << 60) | (1u64 << 61) | (1u64 << 62);

                (rook_square, empty_mask, safe_mask, MoveFlag::KingCastle)
            }

            CastlingRights::BlackQueenside => {
                let rook_square = 56;
                let empty_mask = (1u64 << 57) | (1u64 << 58) | (1u64 << 59);
                let safe_mask = (1u64 << 58) | (1u64 << 59) | (1u64 << 60);

                (rook_square, empty_mask, safe_mask, MoveFlag::QueenCastle)
            }
        };

        // Correct rook must actually exist.
        let rook_board = self.pieces[self.player_turn as usize][PieceKind::Rook as usize];

        if rook_board & (1u64 << rook_square) == 0 {
            println!("rook not found");
            return None;
        }

        // All required squares must be empty.
        if !self.total_occupency & empty_mask != empty_mask {
            println!("squares not empty");
            return None;
        }

        // King cannot currently be in check, cross an attacked square,
        // or land on an attacked square.
        if !self.enemy_attack_mask & safe_mask != safe_mask {
            println!("king in check");
            return None;
        }

        Some(flag)
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

    /// Recalculates the attack mask for the opponent of the current player.
    ///
    /// Generates all pseudo/legal moves available to the opponent and combines
    /// their destination bitboards into a single attack mask.
    ///
    /// The resulting mask is stored in `enemy_attack_mask` and is used,
    /// among other things, when validating king moves and castling.
    fn update_enemy_attack_mask(&mut self) {
        let enemy_possible_moves = self.get_all_legal_moves(self.player_turn, true);
        let mask: BitBoard = enemy_possible_moves
            .iter()
            .fold(BitBoard::EMPTY, |acc, &x| acc | x);

        self.enemy_attack_mask = mask;
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
    fn is_game_over(&self) -> bool {
        let legal_moves = self.get_all_legal_moves(self.player_turn, false);
        let total: u32 = legal_moves.iter().map(|bb| bb.count()).sum();

        total == 0
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
