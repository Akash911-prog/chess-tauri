use crate::engine::movegen::Move;

#[derive(Debug, Clone)]
pub struct HistoryManager {
    undo_struct: Vec<Undo>,
}

impl HistoryManager {
    pub fn new() -> HistoryManager {
        HistoryManager {
            undo_struct: Vec::new(),
        }
    }

    pub fn pop(&mut self) -> Option<Undo> {
        if self.undo_struct.is_empty() {
            return None;
        }
        if let Some(info) = self.undo_struct.pop() {
            return Some(info);
        } else {
            return None;
        }
    }

    pub fn push(&mut self, info: Undo) {
        self.undo_struct.push(info);
    }

    pub fn clone(&self) -> HistoryManager {
        HistoryManager {
            undo_struct: self.undo_struct.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Undo {
    pub mv: Move,              // what was played, so unmake knows how to reverse it
    pub castling_rights: u8,   // rights BEFORE this move (once you've fixed the naming above)
    pub en_passant_square: u8, // ep square BEFORE this move
    pub halfmove_clock: u8,
    pub zobrist_hash: u64, // hash BEFORE this move
}

impl Undo {
    pub fn new(
        mv: Move,
        castling_rights: u8,
        en_passant_square: u8,
        halfmove_clock: u8,
        zobrist_hash: u64,
    ) -> Undo {
        Undo {
            mv,
            castling_rights,
            en_passant_square,
            halfmove_clock,
            zobrist_hash,
        }
    }
}
