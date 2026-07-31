use crate::engine::types::PieceKind;

pub struct HistoryManager {
    undo_struct: Vec<UndoInfo>,
}

pub struct UndoInfo {
    pub move_mask: u16,
    pub captured_piece: PieceKind,
    pub castling_rights: u8,
    pub ep_square: u8,
    pub halfmove_clock: u16,
}

impl HistoryManager {
    pub fn new() -> HistoryManager {
        HistoryManager {
            undo_struct: Vec::new(),
        }
    }

    pub fn pop(&mut self) -> Option<UndoInfo> {
        if self.undo_struct.is_empty() {
            return None;
        }
        if let Some(info) = self.undo_struct.pop() {
            return Some(info);
        } else {
            return None;
        }
    }

    pub fn push(&mut self, info: UndoInfo) {
        self.undo_struct.push(info);
    }
}
