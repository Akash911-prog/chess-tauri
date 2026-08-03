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

    pub _move: Move,

    pub zobrist_hash: u64, // TODO: Add the actualy hash logic and table logic later. Version 2.0

    pub move_gen: MoveGen,
    pub history: HistoryManager,
}

impl Board {
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

            _move: Move::new(0u16, 0u8),

            move_gen: MoveGen::new(),
            history: HistoryManager::new(),
        }
    }

    fn get_color_occupency(&self, color: Color) -> BitBoard {
        self.pieces[color as usize]
            .iter()
            .fold(BitBoard::EMPTY, |a, b| a | *b)
    }

    pub fn get_white_occupency(&self) -> BitBoard {
        self.color_occupency[Color::White]
    }

    pub fn get_black_occupency(&self) -> BitBoard {
        self.color_occupency[Color::Black]
    }

    pub fn get_current_board(&self) -> BitBoard {
        self.total_occupency
    }

    pub fn friendly_pieces(&self) -> BitBoard {
        self.color_occupency[self.player_turn as usize]
    }

    pub fn set_occupency(&mut self) {
        self.color_occupency[0] = self.pieces[0].iter().fold(BitBoard::EMPTY, |a, b| a | *b);
        self.color_occupency[1] = self.pieces[1].iter().fold(BitBoard::EMPTY, |a, b| a | *b);
        self.total_occupency = self.color_occupency[0] | self.color_occupency[1];
    }

    pub fn make_move(&mut self, move_info: Move) {
        let undo = Undo::new(
            self._move,
            self.castling_rights,
            self.en_passant_square,
            self.halfmove_clock,
        );

        self.history.push(undo);

        self._move = move_info;
        self.castling_rights = self.get_castling_rights();
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

        self.set_occupency();

        self.player_turn ^= 1;
    }

    pub fn undo_move(&mut self) {
        let undo = self.history.pop().unwrap();
        self._move = undo.mv;
        self.castling_rights = undo.castling_rights;
        self.en_passant_square = undo.en_passant_square;
        self.halfmove_clock = undo.halfmove_clock;
    }

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
        if !self.validate_move(piece_type, from, to) {
            println!("Invalid move");
            return Ok(Legality::Illegal);
        }

        // --- Castle check & Flag setting ---
        let castle = self.check_castle(piece_type, to);

        match castle {
            Some(flag) => move_mask |= flag.bits(),
            None => {}
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
        self.make_move(new_move);
        Ok(Legality::Legal)
    }

    fn validate_move(&self, piece_type: PieceKind, from: u8, to: u8) -> bool {
        let possible_moves = self.move_gen.get_legal_moves_by_piece(
            piece_type,
            self.total_occupency,
            from as usize,
            self.player_turn,
            self.color_occupency[(self.player_turn ^ 1) as usize],
            self.color_occupency[self.player_turn as usize],
        );

        let current_move = (1u64 << from) | (1u64 << to);

        if let Some(possible_moves) = possible_moves {
            if (possible_moves & current_move) == (1u64 << to) {
                return true;
            }
        }
        false
    }

    fn check_castle(&self, piece_type: PieceKind, to: u8) -> Option<MoveFlag> {
        if piece_type != PieceKind::King {
            return None;
        };

        let casting_rights = match self.check_castling_rights() {
            Some(rights) => rights,
            None => return None,
        };

        if self.player_turn == 0 {
            if (to == 6)
                && ((casting_rights == CastlingRights::Both)
                    || (casting_rights == CastlingRights::King))
            {
                return Some(MoveFlag::KingCastle);
            } else if (to == 2)
                && ((casting_rights == CastlingRights::Both)
                    || (casting_rights == CastlingRights::Queen))
            {
                return Some(MoveFlag::QueenCastle);
            } else {
                return None;
            }
        } else {
            if (to == 57)
                && ((casting_rights == CastlingRights::Both)
                    || (casting_rights == CastlingRights::King))
            {
                return Some(MoveFlag::KingCastle);
            } else if (to == 61)
                && ((casting_rights == CastlingRights::Both)
                    || (casting_rights == CastlingRights::Queen))
            {
                return Some(MoveFlag::QueenCastle);
            } else {
                return None;
            }
        }
    }

    fn check_castling_rights(&self) -> Option<CastlingRights> {
        if self.castling_rights == 0 {
            return None;
        };

        let rights = self.castling_rights & 0x0F;

        if self.player_turn == 0 {
            match rights {
                0x08 => return Some(CastlingRights::King),
                0x04 => return Some(CastlingRights::Queen),
                0x0C => return Some(CastlingRights::Both),
                _ => return None,
            };
        } else {
            match rights {
                0x02 => return Some(CastlingRights::King),
                0x01 => return Some(CastlingRights::Queen),
                0x03 => return Some(CastlingRights::Both),
                _ => return None,
            }
        }
    }

    fn get_castling_rights(&self) -> u8 {
        0x0F
    }

    fn notation_to_index(&self, notation: &str) -> u8 {
        let file = notation.chars().nth(0).unwrap() as u8 - 'a' as u8;
        let rank = notation.chars().nth(1).unwrap() as u8 - '1' as u8;

        if (file > 7) || (rank > 7) {
            return 64; // index beyond 63 means illegal move. That is what is returned
        }
        rank * 8 + file
    }

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

    fn is_game_over(&self) -> bool {
        let legal_moves = self.get_all_legal_moves();
        let total: u32 = legal_moves.iter().map(|bb| bb.count()).sum();

        total == 0
    }

    pub fn get_all_legal_moves(&self) -> Vec<BitBoard> {
        let color = self.player_turn;
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
                ) {
                    all_moves.push(moves);
                }
            }
        }
        all_moves
    }
}
