use crate::dto::PromotionPiece;

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White = 0,
    Black = 1,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastlingRights {
    King,
    Queen,
    Both,
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
        debug_assert!(int < 6);
        PieceKind::from_idx_option(int).unwrap()
    }

    pub fn to_u8(&self) -> u8 {
        *self as u8
    }
}

impl From<PromotionPiece> for PieceKind {
    fn from(p: PromotionPiece) -> Self {
        match p {
            PromotionPiece::Q => PieceKind::Queen,
            PromotionPiece::R => PieceKind::Rook,
            PromotionPiece::B => PieceKind::Bishop,
            PromotionPiece::N => PieceKind::Knight,
        }
    }
}
