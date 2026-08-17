use crate::engine::{
    bitboard::BitBoard,
    board::{moves, Board},
    constants::{
        BISHOP_MOBILITY_EG, BISHOP_MOBILITY_MG, KING_MOBILITY_EG, KING_MOBILITY_MG,
        KNIGHT_MOBILITY_EG, KNIGHT_MOBILITY_MG, MAX_PHASE, PAWN_ADVANCE_BONUS,
        PAWN_BLOCKED_PENALTY, PHASE_WEIGHTS, PST_EG, PST_MG, QUEEN_MOBILITY_EG, QUEEN_MOBILITY_MG,
        ROOK_MOBILITY_EG, ROOK_MOBILITY_MG,
    },
    types::PieceKind,
};

pub struct Evaluator<'a> {
    board: &'a Board,
}

impl<'a> Evaluator<'a> {
    pub fn new(board: &'a Board) -> Evaluator<'a> {
        Evaluator { board }
    }

    pub fn static_eval(&self) -> i32 {
        let phase = self.compute_phase();
        let mut score = [0i32; 2];
        let unoccupied_or_safe = self.board.unoccupied_or_safe();

        // println!("phase: {}", phase);

        for color in 0..2 {
            for piece_idx in 0..6 {
                let mut pieces = self.board.pieces[color][piece_idx];
                while let Some(square) = pieces.pop_lsb() {
                    let sq = if color == 0 { square ^ 56 } else { square }; // flip for white. My board orientation and pesto orientation are inverted. mines a1 their's a8
                    let mg = PST_MG[piece_idx][sq as usize];
                    let eg = PST_EG[piece_idx][sq as usize];
                    let mobility = self.compute_mobility_bonus(
                        PieceKind::from_idx(piece_idx),
                        square as u8,
                        unoccupied_or_safe,
                        phase,
                        sq,
                        color,
                    );
                    score[color] += PieceKind::from_idx(piece_idx).value()
                        + self.interpolate(mg, eg, phase)
                        + mobility;

                    // println!(
                    //     "color: {}, piece: {}, square: {}, mg: {}, eg: {}, phase: {}, score: {}, taper: {}, piece value: {}",
                    //     color, piece_idx, square, mg, eg, phase, score[color], self.taper(mg, eg, phase), PieceKind::from_idx(piece_idx).value()
                    // );
                }
            }
        }

        score[self.board.player_turn as usize] - score[(self.board.player_turn ^ 1) as usize]
    }

    fn compute_mobility_bonus(
        &self,
        piece: PieceKind,
        idx: u8,
        unoccupied_or_safe: BitBoard,
        phase: i32,
        sq: u8,
        color: usize,
    ) -> i32 {
        let moves = match self.board.move_gen.get_legal_moves_by_piece(
            piece,
            self.board.total_occupency,
            idx as usize,
            self.board.player_turn,
            self.board.enemy_attack_mask,
            self.board.color_occupency[self.board.player_turn as usize],
            false,
            self.board.en_passant_square,
            &self.board.kings,
        ) {
            Some(moves) => moves,
            None => BitBoard(0),
        };

        let count = (moves & unoccupied_or_safe).count_ones() as usize;

        let (mg_score, eg_score) = match piece {
            PieceKind::Knight => {
                let i = count.min(KNIGHT_MOBILITY_MG.len() - 1);
                (KNIGHT_MOBILITY_MG[i], KNIGHT_MOBILITY_EG[i])
            }
            PieceKind::Bishop => {
                let i = count.min(BISHOP_MOBILITY_MG.len() - 1);
                (BISHOP_MOBILITY_MG[i], BISHOP_MOBILITY_EG[i])
            }
            PieceKind::Rook => {
                let i = count.min(ROOK_MOBILITY_MG.len() - 1);
                (ROOK_MOBILITY_MG[i], ROOK_MOBILITY_EG[i])
            }
            PieceKind::Queen => {
                let i = count.min(QUEEN_MOBILITY_MG.len() - 1);
                (QUEEN_MOBILITY_MG[i], QUEEN_MOBILITY_EG[i])
            }
            PieceKind::King => {
                let i = count.min(KING_MOBILITY_MG.len() - 1);
                (KING_MOBILITY_MG[i], KING_MOBILITY_EG[i])
            }
            PieceKind::Pawn => return self.compute_pawn_bonus(sq, moves, color),
        };

        self.interpolate(mg_score, eg_score, phase)
    }

    fn compute_pawn_bonus(&self, sq: u8, moves: BitBoard, color: usize) -> i32 {
        let rank = if color == 0 { sq / 8u8 } else { 7 - sq / 8u8 }; // flip for black
        let is_blocked = ((moves & !self.board.total_occupency) == 0) as i32;

        PAWN_ADVANCE_BONUS[rank as usize] + (PAWN_BLOCKED_PENALTY[rank as usize] * is_blocked)
    }

    /// Returns a phase value from 0 (endgame, minimal material)
    /// to MAX_PHASE (opening/midgame, full material).
    fn compute_phase(&self) -> i32 {
        let mut phase = 0;
        for color in 0..2 {
            for piece_idx in 0..6 {
                let count = self.board.pieces[color][piece_idx].count_ones() as i32;
                phase += count * PHASE_WEIGHTS[piece_idx];
            }
        }
        phase.min(MAX_PHASE) // clamp in case of promotions creating extra queens/etc.
    }

    /// Blends a midgame and endgame score based on current phase.
    /// phase = MAX_PHASE -> fully mg_score. phase = 0 -> fully eg_score.
    fn interpolate(&self, mg_score: i32, eg_score: i32, phase: i32) -> i32 {
        (mg_score * phase + eg_score * (MAX_PHASE - phase)) / MAX_PHASE
    }
}
