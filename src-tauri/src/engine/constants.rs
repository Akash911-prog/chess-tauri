use crate::engine::bitboard::BitBoard;

// File masks to prevent board wrap-around during shifts
pub const FILE_A: u64 = 0x0101_0101_0101_0101;
pub const FILE_B: u64 = FILE_A << 1;
pub const FILE_G: u64 = FILE_A << 6;
pub const FILE_H: u64 = FILE_A << 7;

pub const FILE_AB: u64 = FILE_A | FILE_B;
pub const FILE_GH: u64 = FILE_G | FILE_H;

pub const INITIAL_BOARD: [[BitBoard; 6]; 2] = [
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

pub const W_QUEEN_CASTLE: u64 = 0x0000_0000_0000_0014;
pub const W_KING_CASTLE: u64 = 0x0000_0000_0000_0050;

pub const B_QUEEN_CASTLE: u64 = 0x1400_0000_0000_0000;
pub const B_KING_CASTLE: u64 = 0x5000_0000_0000_0000;
