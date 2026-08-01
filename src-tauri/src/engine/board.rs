use crate::{
    dto::{CommandError, Legality, MoveInfo},
    engine::{
        bitboard::BitBoard,
        history::HistoryManager,
        movegen::{Move, MoveGen},
        types::{Color, PieceKind},
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

    pub player_turn: u8, // 0 = white, 1 = black
    pub castling_rights: u8,
    pub promotion: u8,         // KQkq
    pub en_passant_square: u8, // index of sq
    pub halfmove_clock: u16,   // clock
    pub fullmove_clock: u16,

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
            castling_rights: 0,
            promotion: 0,
            en_passant_square: 0,
            halfmove_clock: 0,
            fullmove_clock: 0,

            zobrist_hash: 0,

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

    pub fn make_move(&mut self, move_info: Move) -> Legality {
        Legality::Legal
    }

    pub fn undo_move(&mut self) {}

    pub fn parse_react_move(&mut self, move_info: MoveInfo) -> Result<Legality, CommandError> {
        let new_move: Move;
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
        let move_mask = ((to as u16) << 6) | (from as u16);
        let piece_mask = 1u64 << from;
        let captured_piece_mask = 1u64 << to;
        let side = self.player_turn;

        // Find the piece index (0 = Pawn, 1 = Knight, etc.) that occupies the 'from' square
        let piece_idx = self.get_piece_index(piece_mask, side);

        if piece_idx > 5 {
            return Err(CommandError::EmptySquare { square: from });
        }
        let piece_type = PieceKind::from_idx(piece_idx);

        if !(self.validate_move(piece_type, from, to)) {
            return Ok(Legality::Illegal);
        }

        if (self.color_occupency[!side as usize] & captured_piece_mask) == 0 {
            let piece = (6u8 << 4) | (piece_idx as u8);
            new_move = Move::new(move_mask, piece);
        } else {
            let captured_piece_idx = self.get_piece_index(captured_piece_mask, !side);

            if captured_piece_idx > 5 {
                return Ok(Legality::Illegal);
            };

            let captured_piece = ((captured_piece_idx as u8) << 4) | (captured_piece_idx as u8);
            new_move = Move::new(move_mask, captured_piece);
        }

        Ok(self.make_move(new_move))
    }

    fn validate_move(&self, piece_type: PieceKind, from: u8, to: u8) -> bool {
        let possible_moves = self.move_gen.get_legal_moves_by_piece(
            piece_type,
            self.total_occupency,
            from as usize,
            self.player_turn,
            self.color_occupency[!self.player_turn as usize],
            self.color_occupency[self.player_turn as usize],
        );

        let current_move = (from as u64) | (to as u64);

        if let Some(possible_moves) = possible_moves {
            if (possible_moves & current_move) == current_move {
                return true;
            }
        }
        false
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
}
