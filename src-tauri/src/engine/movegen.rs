use std::fs::OpenOptions;
use std::io::Write;

use crate::engine::{
    bitboard::BitBoard,
    constants::{FILE_A, FILE_AB, FILE_GH, FILE_H},
    types::PieceKind::{self, Bishop, King, Knight, Pawn, Queen, Rook},
};

pub struct MoveGen {
    knight_moves: [u64; 64],
    king_moves: [u64; 64],
    pawn_attack: [[u64; 64]; 2],
    pawn_push_single: [[u64; 64]; 2],
    pawn_push_double: [[u64; 64]; 2],
}

impl MoveGen {
    pub fn new() -> MoveGen {
        let gen = MoveGen {
            knight_moves: [0; 64],
            king_moves: [0; 64],
            pawn_attack: [[0; 64]; 2], // Fixed syntax: semicolon instead of comma
            pawn_push_single: [[0; 64]; 2],
            pawn_push_double: [[0; 64]; 2],
        };

        gen
    }

    pub fn generate_moves(&mut self) {
        // Populate all tables on creation
        self.gen_knight_moves();
        self.gen_king_moves();
        self.gen_pawn_attacks();
        self.gen_pawn_pushes();
    }

    pub fn get_legal_moves_by_piece(
        &self,
        piece: PieceKind,
        occupied: BitBoard,
        idx: usize,
        color: u8,
        enemy: BitBoard,
        friendly: BitBoard,
        attack_only: bool,
        ep_square: u8,
    ) -> Option<BitBoard> {
        if attack_only {
            return match piece {
                Knight => self.get_knight_moves(idx, friendly, attack_only),
                King => self.get_king_attacks(idx, friendly),
                Pawn => self.get_pawn_attacks(idx, color, occupied, enemy, attack_only, ep_square),
                Rook => Some(BitBoard(self.gen_rook_attacks(idx, occupied))),
                Bishop => Some(BitBoard(self.gen_bishop_attacks(idx, occupied))),
                Queen => Some(BitBoard(self.gen_queen_attacks(idx, occupied))),
            };
        }
        match piece {
            Knight => self.get_knight_moves(idx, friendly, attack_only),
            King => self.get_king_attacks(idx, friendly),
            Pawn => self.get_pawn_attacks(idx, color, occupied, enemy, attack_only, ep_square),
            Rook => self.get_rook_moves(idx, occupied, friendly),
            Bishop => self.get_bishop_moves(idx, occupied, friendly),
            Queen => self.get_queen_moves(idx, occupied, friendly),
        }
    }

    pub fn get_knight_moves(
        &self,
        idx: usize,
        friendly: BitBoard,
        attack_only: bool,
    ) -> Option<BitBoard> {
        let possible_moves = if attack_only {
            BitBoard(self.knight_moves[idx])
        } else {
            self.knight_moves[idx] & !friendly
        };
        Some(possible_moves)
    }

    pub fn get_king_attacks(&self, idx: usize, friendly: BitBoard) -> Option<BitBoard> {
        let possible_moves = self.king_moves[idx] & !friendly;
        Some(possible_moves)
    }

    pub fn get_pawn_attacks(
        &self,
        idx: usize,
        color: u8,
        occupied: BitBoard,
        enemy: BitBoard,
        attack_only: bool,
        ep_square: u8,
    ) -> Option<BitBoard> {
        let single = self.pawn_push_single[color as usize][idx] & !occupied;

        let mut moves = single;

        if single != 0 {
            let double = self.pawn_push_double[color as usize][idx] & !occupied;

            moves |= double;
        }

        moves |= self.pawn_attack[color as usize][idx] & enemy;
        if (ep_square < 64)
            && ((self.pawn_attack[color as usize][idx] & BitBoard(1u64 << ep_square)) != 0)
        {
            moves |= BitBoard(1u64 << ep_square);
        }

        if attack_only {
            return Some(BitBoard(self.pawn_attack[color as usize][idx]));
        }
        return Some(moves);
    }

    fn _debug(&self, color: u8) {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("debug_bitboards.txt")
            .expect("failed to open debug log file");

        writeln!(file, "=== pawn_push_single (color {}) ===", color).unwrap();
        self.pawn_push_single[color as usize]
            .iter()
            .for_each(|&bb| writeln!(file, "{}", BitBoard(bb)).unwrap());

        writeln!(file, "=== pawn_push_double (color {}) ===", color).unwrap();
        self.pawn_push_double[color as usize]
            .iter()
            .for_each(|&bb| writeln!(file, "{}", BitBoard(bb)).unwrap());

        writeln!(file, "=== pawn_attack (color {}) ===", color).unwrap();
        self.pawn_attack[color as usize]
            .iter()
            .for_each(|&bb| writeln!(file, "{}", BitBoard(bb)).unwrap());
    }

    pub fn gen_knight_moves(&mut self) {
        for sq in 0..64 {
            let n = 1u64 << sq;
            let mut attacks = 0u64;

            // Up/Down 2, Left/Right 1 (Requires NOT_A or NOT_H)
            attacks |= (n.wrapping_shl(17)) & !FILE_A;
            attacks |= (n.wrapping_shl(15)) & !FILE_H;
            attacks |= (n.wrapping_shr(15)) & !FILE_A;
            attacks |= (n.wrapping_shr(17)) & !FILE_H;

            // Up/Down 1, Left/Right 2 (Requires NOT_AB or NOT_GH)
            attacks |= (n.wrapping_shl(10)) & !FILE_AB;
            attacks |= (n.wrapping_shl(6)) & !FILE_GH;
            attacks |= (n.wrapping_shr(6)) & !FILE_AB;
            attacks |= (n.wrapping_shr(10)) & !FILE_GH;

            self.knight_moves[sq] = attacks;
        }
    }

    pub fn gen_king_moves(&mut self) {
        for sq in 0..64 {
            let k = 1u64 << sq;
            let mut attacks = 0u64;

            attacks |= k.wrapping_shl(8); // Up
            attacks |= k.wrapping_shr(8); // Down
            attacks |= (k.wrapping_shr(1)) & !FILE_A; // Left
            attacks |= (k.wrapping_shl(1)) & !FILE_H; // Right
            attacks |= (k.wrapping_shl(7)) & !FILE_H; // Up-Left
            attacks |= (k.wrapping_shl(9)) & !FILE_A; // Up-Right
            attacks |= (k.wrapping_shr(9)) & !FILE_H; // Down-Left
            attacks |= (k.wrapping_shr(7)) & !FILE_A; // Down-Right

            self.king_moves[sq] = attacks;
        }
    }

    fn gen_pawn_attacks(&mut self) {
        for sq in 0..64 {
            let p = 1u64 << sq;

            // WHITE PAWN ATTACKS (Up: 0)
            self.pawn_attack[0][sq] = (p.wrapping_shl(7) & !FILE_H) | (p.wrapping_shl(9) & !FILE_A);

            // BLACK PAWN ATTACKS (Down: 1)
            self.pawn_attack[1][sq] = (p.wrapping_shr(9) & !FILE_H) | (p.wrapping_shr(7) & !FILE_A);
        }
    }

    fn gen_pawn_pushes(&mut self) {
        for sq in 0..64 {
            let p = 1u64 << sq;

            // WHITE PUSHES (Index 0)
            self.pawn_push_single[0][sq] = p.wrapping_shl(8);

            // Only rank 2 pawns (squares 8..15) can double push to rank 4
            if (8..=15).contains(&sq) {
                self.pawn_push_double[0][sq] = p.wrapping_shl(16);
            }

            // BLACK PUSHES (Index 1)
            self.pawn_push_single[1][sq] = p.wrapping_shr(8);

            // Only rank 7 pawns (squares 48..55) can double push to rank 5
            if (48..=55).contains(&sq) {
                self.pawn_push_double[1][sq] = p.wrapping_shr(16);
            }
        }
    }

    pub fn gen_rook_attacks(&self, sq: usize, occupied: BitBoard) -> u64 {
        let mut attacks = 0u64;

        // 1. UP (+8)
        let mut r_sq = sq;
        while r_sq < 56 {
            // Can't go higher than rank 7 before stepping
            r_sq += 8;
            let bit = 1u64 << r_sq;
            attacks |= bit;
            if (bit & occupied) != 0 {
                break;
            } // Hit a blocker!
        }

        // 2. DOWN (-8)
        r_sq = sq;
        while r_sq >= 8 {
            // Can't go lower than rank 2 before stepping
            r_sq -= 8;
            let bit = 1u64 << r_sq;
            attacks |= bit;
            if (bit & occupied) != 0 {
                break;
            }
        }

        // 3. RIGHT (+1)
        r_sq = sq;
        while (r_sq % 8) != 7 {
            // Stop if on File H
            r_sq += 1;
            let bit = 1u64 << r_sq;
            attacks |= bit;
            if (bit & occupied) != 0 {
                break;
            }
        }

        // 4. LEFT (-1)
        r_sq = sq;
        while (r_sq % 8) != 0 {
            // Stop if on File A
            r_sq -= 1;
            let bit = 1u64 << r_sq;
            attacks |= bit;
            if (bit & occupied) != 0 {
                break;
            }
        }

        attacks
    }

    /// Generates attacks for a Bishop on `sq` considering all `occupied` pieces on the board.
    pub fn gen_bishop_attacks(&self, sq: usize, occupied: BitBoard) -> u64 {
        let mut attacks = 0u64;

        // 1. UP-RIGHT (+9)
        let mut r_sq = sq;
        while r_sq < 56 && (r_sq % 8) != 7 {
            r_sq += 9;
            let bit = 1u64 << r_sq;
            attacks |= bit;
            if (bit & occupied) != 0 {
                break;
            }
        }

        // 2. UP-LEFT (+7)
        r_sq = sq;
        while r_sq < 56 && (r_sq % 8) != 0 {
            r_sq += 7;
            let bit = 1u64 << r_sq;
            attacks |= bit;
            if (bit & occupied) != 0 {
                break;
            }
        }

        // 3. DOWN-RIGHT (-7)
        r_sq = sq;
        while r_sq >= 8 && (r_sq % 8) != 7 {
            r_sq -= 7;
            let bit = 1u64 << r_sq;
            attacks |= bit;
            if (bit & occupied) != 0 {
                break;
            }
        }

        // 4. DOWN-LEFT (-9)
        r_sq = sq;
        while r_sq >= 8 && (r_sq % 8) != 0 {
            r_sq -= 9;
            let bit = 1u64 << r_sq;
            attacks |= bit;
            if (bit & occupied) != 0 {
                break;
            }
        }

        attacks
    }

    /// Queen attacks are simply Rook attacks OR Bishop attacks!
    pub fn gen_queen_attacks(&self, sq: usize, occupied: BitBoard) -> u64 {
        self.gen_rook_attacks(sq, occupied) | self.gen_bishop_attacks(sq, occupied)
    }

    // board/check.rs, or a new board/pins.rs — your call
    // in movegen.rs, alongside gen_rook_attacks etc.
    pub fn ray_between(&self, from: usize, to: usize) -> BitBoard {
        let (ff, fr) = (from as i8 % 8, from as i8 / 8);
        let (tf, tr) = (to as i8 % 8, to as i8 / 8);
        let (df, dr) = (tf - ff, tr - fr);

        let dir = match (df.signum(), dr.signum()) {
            (0, 0) => return BitBoard(0),
            (d, 0) if df != 0 => (d, 0),
            (0, d) if dr != 0 => (0, d),
            (dx, dy) if df.abs() == dr.abs() => (dx, dy),
            _ => return BitBoard(0), // not aligned
        };

        let mut squares = 0u64;
        let (mut f, mut r) = (ff + dir.0, fr + dir.1);
        while (f, r) != (tf, tr) {
            squares |= 1u64 << (r * 8 + f);
            f += dir.0;
            r += dir.1;
        }
        BitBoard(squares)
    }

    pub fn get_bishop_moves(
        &self,
        idx: usize,
        occupied: BitBoard,
        friendly: BitBoard,
    ) -> Option<BitBoard> {
        Some(self.gen_bishop_attacks(idx, occupied) & !friendly)
    }

    pub fn get_rook_moves(
        &self,
        idx: usize,
        occupied: BitBoard,
        friendly: BitBoard,
    ) -> Option<BitBoard> {
        Some(self.gen_rook_attacks(idx, occupied) & !friendly)
    }

    pub fn get_queen_moves(
        &self,
        idx: usize,
        occupied: BitBoard,
        friendly: BitBoard,
    ) -> Option<BitBoard> {
        Some(self.gen_queen_attacks(idx, occupied) & !friendly)
    }
}

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveFlag {
    Quiet = 0b0000,
    DoublePush = 0b0001,
    KingCastle = 0b0010,
    QueenCastle = 0b0011,
    Capture = 0b0100,
    EpCapture = 0b0101,
    PromoQueen = 0b1000,
    PromoRook = 0b1001,
    PromoKnight = 0b1010,
    PromoBishop = 0b1011,
}

impl MoveFlag {
    /// Shifts this flag into bits 12..15 for packing into a move u16.
    #[inline(always)]
    pub fn bits(self) -> u16 {
        (self as u16) << 12
    }

    #[inline(always)]
    pub fn from_bits(bits: u16) -> MoveFlag {
        match bits >> 12 {
            0b0000 => MoveFlag::Quiet,
            0b0001 => MoveFlag::DoublePush,
            0b0010 => MoveFlag::KingCastle,
            0b0011 => MoveFlag::QueenCastle,
            0b0100 => MoveFlag::Capture,
            0b0101 => MoveFlag::EpCapture,
            0b1000 => MoveFlag::PromoQueen,
            0b1001 => MoveFlag::PromoRook,
            0b1010 => MoveFlag::PromoKnight,
            0b1011 => MoveFlag::PromoBishop,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Move {
    // board State
    move_mask: u16, // 16-bit move encoding: [6-bit from][6-bit to][4-bit special flags]
    piece: u8,      // 4-bit piece type ([4-bit piece][4-bit captured piece]) value > 6 means None
}

impl Move {
    pub fn new(move_mask: u16, piece: u8) -> Move {
        Move { move_mask, piece }
    }

    pub fn from(&self) -> u8 {
        (self.move_mask & 0x3F) as u8
    }

    pub fn to(&self) -> u8 {
        ((self.move_mask >> 6) & 0x3F) as u8
    }

    pub fn flags(&self) -> u16 {
        self.move_mask & 0xF000
    }

    pub fn piece(&self) -> u8 {
        (self.piece & 0x0F) as u8
    }

    pub fn captured_piece(&self) -> u8 {
        (self.piece >> 4) as u8
    }
}
