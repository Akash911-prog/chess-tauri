use std::fmt;
use std::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Index, IndexMut, Not, Shl,
    ShlAssign, Shr, ShrAssign,
};

use crate::engine::types::{Color, PieceKind};

/// Represents a 64-square chessboard as a 64-bit integer bitboard.
/// Bit 0 = A1, Bit 63 = H8.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct BitBoard(pub u64);

// File masks to prevent board wrap-around during shifts
pub const FILE_A: u64 = 0x0101_0101_0101_0101;
pub const FILE_B: u64 = FILE_A << 1;
pub const FILE_G: u64 = FILE_A << 6;
pub const FILE_H: u64 = FILE_A << 7;

pub const FILE_AB: u64 = FILE_A | FILE_B;
pub const FILE_GH: u64 = FILE_G | FILE_H;

impl BitBoard {
    pub const EMPTY: Self = BitBoard(0);
    pub const ALL: Self = BitBoard(u64::MAX);

    #[inline(always)]
    pub fn value(&self) -> u64 {
        self.0
    }

    /// Construct a BitBoard with a single bit set at `index` (0..63).
    #[inline(always)]
    pub const fn from_square(index: u8) -> Self {
        BitBoard(1u64 << index)
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    #[inline(always)]
    pub fn is_set(&self, index: u8) -> bool {
        (self.0 & (1u64 << index)) != 0
    }

    #[inline(always)]
    pub fn set(&mut self, index: u8) {
        self.0 |= 1u64 << index;
    }

    #[inline(always)]
    pub fn clear(&mut self, index: u8) {
        self.0 &= !(1u64 << index);
    }

    #[inline(always)]
    pub fn toggle(&mut self, index: u8) {
        self.0 ^= 1u64 << index;
    }

    #[inline(always)]
    pub fn count(&self) -> u32 {
        self.0.count_ones()
    }

    /// Gets the index of the Least Significant Bit (LSB) without clearing it.
    #[inline(always)]
    pub fn lsb(&self) -> u8 {
        self.0.trailing_zeros() as u8
    }

    /// Pops and returns the index of the Least Significant Bit (LSB).
    #[inline(always)]
    pub fn pop_lsb(&mut self) -> Option<u8> {
        if self.0 == 0 {
            return None;
        }
        let sq = self.lsb();
        self.0 &= self.0 - 1; // Brian Kernighan's trick
        Some(sq)
    }

    // Directional shifts operating directly on the BitBoard struct
    #[inline(always)]
    pub fn shift_north(self) -> Self {
        BitBoard(self.0 << 8)
    }
    #[inline(always)]
    pub fn shift_south(self) -> Self {
        BitBoard(self.0 >> 8)
    }
    #[inline(always)]
    pub fn shift_east(self) -> Self {
        BitBoard((self.0 & !FILE_H) << 1)
    }
    #[inline(always)]
    pub fn shift_west(self) -> Self {
        BitBoard((self.0 & !FILE_A) >> 1)
    }
    #[inline(always)]
    pub fn shift_ne(self) -> Self {
        BitBoard((self.0 & !FILE_H) << 9)
    }
    #[inline(always)]
    pub fn shift_nw(self) -> Self {
        BitBoard((self.0 & !FILE_A) << 7)
    }
    #[inline(always)]
    pub fn shift_se(self) -> Self {
        BitBoard((self.0 & !FILE_H) >> 7)
    }
    #[inline(always)]
    pub fn shift_sw(self) -> Self {
        BitBoard((self.0 & !FILE_A) >> 9)
    }
}

// ----------------------------------------------------------------------------
// Bitboard Indexing
// ----------------------------------------------------------------------------

impl Index<Color> for [BitBoard; 2] {
    type Output = BitBoard;

    #[inline(always)]
    fn index(&self, color: Color) -> &Self::Output {
        &self[color as usize]
    }
}

// Allows writing: board.occupancies[Color::White] = ...
impl IndexMut<Color> for [BitBoard; 2] {
    #[inline(always)]
    fn index_mut(&mut self, color: Color) -> &mut Self::Output {
        &mut self[color as usize]
    }
}

impl Index<PieceKind> for [BitBoard; 6] {
    type Output = BitBoard;

    #[inline(always)]
    fn index(&self, piece: PieceKind) -> &Self::Output {
        &self[piece as usize]
    }
}

impl IndexMut<PieceKind> for [BitBoard; 6] {
    #[inline(always)]
    fn index_mut(&mut self, piece: PieceKind) -> &mut Self::Output {
        &mut self[piece as usize]
    }
}

// ----------------------------------------------------------------------------
// Iteration Support
// Allows writing: `for sq in bitboard { ... }`
// ----------------------------------------------------------------------------

impl Iterator for BitBoard {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.pop_lsb()
    }
}

// ----------------------------------------------------------------------------
// Bitwise Operator Implementations (BitBoard <-> BitBoard)
// ----------------------------------------------------------------------------

impl BitOr for BitBoard {
    type Output = Self;
    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self {
        BitBoard(self.0 | rhs.0)
    }
}

impl BitAnd for BitBoard {
    type Output = Self;
    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self {
        BitBoard(self.0 & rhs.0)
    }
}

impl BitXor for BitBoard {
    type Output = Self;
    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self {
        BitBoard(self.0 ^ rhs.0)
    }
}

impl Not for BitBoard {
    type Output = Self;
    #[inline(always)]
    fn not(self) -> Self {
        BitBoard(!self.0)
    }
}

// ----------------------------------------------------------------------------
// Bitwise Operator Implementations with raw u64 (BitBoard <-> u64)
// Allows seamlessly doing: `bitboard & 0x00FF` or `bitboard | mask`
// ----------------------------------------------------------------------------

impl BitOr<u64> for BitBoard {
    type Output = Self;
    #[inline(always)]
    fn bitor(self, rhs: u64) -> Self {
        BitBoard(self.0 | rhs)
    }
}

impl BitAnd<u64> for BitBoard {
    type Output = Self;
    #[inline(always)]
    fn bitand(self, rhs: u64) -> Self {
        BitBoard(self.0 & rhs)
    }
}

impl BitXor<u64> for BitBoard {
    type Output = Self;
    #[inline(always)]
    fn bitxor(self, rhs: u64) -> Self {
        BitBoard(self.0 ^ rhs)
    }
}

// ----------------------------------------------------------------------------
// Assign Operators (`|=`, `&=`, `^=`, `<<=`, `>>=`)
// ----------------------------------------------------------------------------

impl BitOrAssign for BitBoard {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitOrAssign<u64> for BitBoard {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: u64) {
        self.0 |= rhs;
    }
}

impl BitAndAssign for BitBoard {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitAndAssign<u64> for BitBoard {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: u64) {
        self.0 &= rhs;
    }
}

impl BitXorAssign for BitBoard {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl BitXorAssign<u64> for BitBoard {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: u64) {
        self.0 ^= rhs;
    }
}

// ----------------------------------------------------------------------------
// Shifts
// ----------------------------------------------------------------------------

impl Shl<u8> for BitBoard {
    type Output = Self;
    #[inline(always)]
    fn shl(self, rhs: u8) -> Self {
        BitBoard(self.0 << rhs)
    }
}

impl Shr<u8> for BitBoard {
    type Output = Self;
    #[inline(always)]
    fn shr(self, rhs: u8) -> Self {
        BitBoard(self.0 >> rhs)
    }
}

impl ShlAssign<u8> for BitBoard {
    #[inline(always)]
    fn shl_assign(&mut self, rhs: u8) {
        self.0 <<= rhs;
    }
}

impl ShrAssign<u8> for BitBoard {
    #[inline(always)]
    fn shr_assign(&mut self, rhs: u8) {
        self.0 >>= rhs;
    }
}

// ----------------------------------------------------------------------------
// Convenience Traits (Display & From/Into)
// ----------------------------------------------------------------------------

impl From<u64> for BitBoard {
    #[inline(always)]
    fn from(val: u64) -> Self {
        BitBoard(val)
    }
}

impl From<BitBoard> for u64 {
    #[inline(always)]
    fn from(bb: BitBoard) -> Self {
        bb.0
    }
}

/// Formats the bitboard as a neat 8x8 ASCII grid when printed with `{}`
impl fmt::Display for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "  +-----------------+",)?;
        for rank in (0..8).rev() {
            write!(f, "{} | ", rank + 1)?;
            for file in 0..8 {
                let square = rank * 8 + file;
                if self.is_set(square) {
                    write!(f, "1 ")?;
                } else {
                    write!(f, ". ")?;
                }
            }
            writeln!(f, "|")?;
        }
        writeln!(f, "  +-----------------+")?;
        writeln!(f, "    a b c d e f g h")
    }
}
