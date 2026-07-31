use crate::engine::{bitboard::BitBoard, types::Color};

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
}
