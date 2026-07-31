pub struct MoveGen {
    knight_moves: [u64; 64],
    king_moves: [u64; 64],
    pawn_attack: [u64; 64],
    pawn_push_single: [u64; 64],
    pawn_push_double: [u64; 64],
}

impl MoveGen {
    pub fn new() -> MoveGen {
        MoveGen {
            knight_moves: [0; 64],
            king_moves: [0; 64],
            pawn_attack: [0; 64],
            pawn_push_single: [0; 64],
            pawn_push_double: [0; 64],
        }
    }
}
