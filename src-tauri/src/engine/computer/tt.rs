use crate::engine::movegen::Move;

#[derive(Clone, Copy)]
pub enum Bound {
    Exact,
    Lower,
    Upper,
}

#[derive(Clone, Copy)]
pub struct TTEntry {
    pub key: u64,
    pub depth: u8,
    pub score: i32,
    pub bound: Bound,
    pub best_move: Option<Move>,
}

pub struct TranspositionTable {
    entries: Vec<Option<TTEntry>>,
}

impl TranspositionTable {
    pub fn new(size_mb: usize) -> Self {
        let entry_size = std::mem::size_of::<Option<TTEntry>>();
        let count = (size_mb * 1024 * 1024) / entry_size;

        Self {
            entries: vec![None; count.max(1)],
        }
    }

    #[inline(always)]
    fn index(&self, key: u64) -> usize {
        key as usize % self.entries.len()
    }

    #[inline(always)]
    pub fn probe(&self, key: u64) -> Option<TTEntry> {
        let entry = self.entries[self.index(key)]?;

        if entry.key == key {
            Some(entry)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn store(&mut self, entry: TTEntry) {
        let index = self.index(entry.key);

        match self.entries[index] {
            None => {
                self.entries[index] = Some(entry);
            }

            Some(old) if entry.depth >= old.depth => {
                self.entries[index] = Some(entry);
            }

            _ => {}
        }
    }
}
