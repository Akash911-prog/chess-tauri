use serde::{Deserialize, Serialize};

use crate::dto::PromotionPiece;

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    pub fn from(num: u8) -> Color {
        match num {
            0 => Color::White,
            _ => Color::Black,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceKind {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastlingRights {
    WhiteQueenside = 0x04,
    WhiteKingside = 0x08,

    BlackQueenside = 0x01,
    BlackKingside = 0x02,
}

impl CastlingRights {
    pub fn bits(self) -> u8 {
        self as u8
    }
}

impl PieceKind {
    fn from_idx_option(int: usize) -> Option<PieceKind> {
        match int {
            0 => Some(PieceKind::Pawn),
            1 => Some(PieceKind::Knight),
            2 => Some(PieceKind::Bishop),
            3 => Some(PieceKind::Rook),
            4 => Some(PieceKind::Queen),
            5 => Some(PieceKind::King),
            _ => None,
        }
    }

    pub fn from_idx(int: usize) -> PieceKind {
        if int >= 6 {
            return PieceKind::Pawn;
        }
        PieceKind::from_idx_option(int).unwrap()
    }

    pub fn to_u8(&self) -> u8 {
        *self as u8
    }
}

impl From<PromotionPiece> for PieceKind {
    fn from(p: PromotionPiece) -> Self {
        match p {
            PromotionPiece::Queen => PieceKind::Queen,
            PromotionPiece::Rook => PieceKind::Rook,
            PromotionPiece::Bishop => PieceKind::Bishop,
            PromotionPiece::Knight => PieceKind::Knight,
        }
    }
}
