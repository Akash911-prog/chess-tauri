use crate::{
    dto::MoveInfo,
    engine::{
        bitboard::BitBoard,
        history::{HistoryManager, UndoInfo},
        movegen::MoveGen,
        types::Color,
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

    // board State
    pub player_turn: u8,     // 0 = white, 1 = black
    pub castling_rights: u8, // KQkq
    pub en_passant: u8,      // index of sq
    pub halfmove_clock: u16, // clock
    pub fullmove_clock: u16, // move counter

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
            castling_rights: 0b1111,
            en_passant: 0,
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

    pub fn make_move(&mut self, move_info: MoveInfo) {
        let from = self.notation_to_index(&move_info.from);
        let to = self.notation_to_index(&move_info.to);

        // 16-bit move encoding: [6-bit from][6-bit to]
        let move_mask = ((from as u16) << 8) | (to as u16);
        let piece_mask = 1u64 << from;

        let side = self.player_turn as usize;

        // Find the piece index (0 = Pawn, 1 = Knight, etc.) that occupies the 'from' square
        let piece_type = self.pieces[side]
            .iter()
            .position(|&piece_bitboard| (piece_bitboard & piece_mask) != BitBoard(0));

        match piece_type {
            Some(piece_index) => {
                // Found your piece! (e.g., piece_index = 1 for Knight)
                println!("Moving piece type {}", piece_index);

                // Update bitboard: clear 'from' bit, set 'to' bit
                self.pieces[side][piece_index] ^= piece_mask; // Remove from 'from'
                self.pieces[side][piece_index] |= 1u64 << to; // Add to 'to'
            }
            None => panic!("Illegal move: No piece found on square {}", move_info.from),
        }
    }

    fn notation_to_index(&self, notation: &str) -> u8 {
        let file = notation.chars().nth(0).unwrap() as u8 - 'a' as u8;
        let rank = notation.chars().nth(1).unwrap() as u8 - '1' as u8;
        rank * 8 + file
    }
}
