use crate::engine::{
    bitboard::{BitBoard, FILE_A, FILE_AB, FILE_GH, FILE_H},
    types::PieceKind,
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

            attacks |= (k.wrapping_shl(8)); // Up
            attacks |= (k.wrapping_shr(8)); // Down
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

    pub fn get_rook_attacks(&self, sq: usize, occupied: u64) -> u64 {
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
    pub fn get_bishop_attacks(&self, sq: usize, occupied: u64) -> u64 {
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
    pub fn get_queen_attacks(&self, sq: usize, occupied: u64) -> u64 {
        self.get_rook_attacks(sq, occupied) | self.get_bishop_attacks(sq, occupied)
    }
}

// Flag definitions for Bits 12..15
pub const FLAG_QUIET: u16 = 0b0000 << 12;
pub const FLAG_DOUBLE_PUSH: u16 = 0b0001 << 12;
pub const FLAG_KING_CASTLE: u16 = 0b0010 << 12;
pub const FLAG_QUEEN_CASTLE: u16 = 0b0011 << 12;
pub const FLAG_CAPTURE: u16 = 0b0100 << 12;
pub const FLAG_EP_CAPTURE: u16 = 0b0101 << 12;
pub const FLAG_PROMO_QUEEN: u16 = 0b1000 << 12;
pub const FLAG_PROMO_ROOK: u16 = 0b1001 << 12;

#[derive(Debug, Clone, Copy)]
pub struct Move {
    // board State
    move_mask: u16, // 16-bit move encoding: [6-bit from][6-bit to][4-bit special flags]
    piece: u8,      // 4-bit piece type ([4-bit piece][4-bit captured piece]) value > 7 means None
}

impl Move {
    pub fn new(move_mask: u16, piece: u8) -> Move {
        Move { move_mask, piece }
    }

    pub fn move_mask(&self) -> u16 {
        self.move_mask
    }

    pub fn from(&self) -> u8 {
        (self.move_mask & 0x3F) as u8
    }

    pub fn to(&self) -> u8 {
        ((self.move_mask >> 6) & 0x3F) as u8
    }

    pub fn flags(&self) -> u8 {
        (self.move_mask & 0xF000) as u8
    }
}
