use crate::engine::{
    board::Board,
    computer::evaluator::Evaluator,
    constants::MATE_SCORE,
    movegen::{Move, MoveFlag},
    types::PieceKind,
};

pub struct Search<'a> {
    board: &'a mut Board,
    pub nodes_visited: u64,
    // later: tt: TranspositionTable, killers: [[Move; 2]; MAX_DEPTH], stop_time: Instant, ...
}

impl<'a> Search<'a> {
    pub fn new(board: &'a mut Board) -> Self {
        Search {
            board,
            nodes_visited: 0,
        }
    }

    pub fn negamax(&mut self, depth: u8, ply: i32, mut alpha: i32, beta: i32) -> i32 {
        self.nodes_visited += 1;

        let mut legal_moves = self.board.generate_legal_moves();

        legal_moves.sort_unstable_by_key(|mv| std::cmp::Reverse(self.move_order_score(*mv)));

        if legal_moves.is_empty() {
            if self.board.check_for_check().is_check {
                return -(self.mate_score(ply));
            }
            return 0;
        }

        if depth == 0 {
            return self.quiescence(alpha, beta);
        }

        let mut best_score = i32::MIN + 1;

        for mv in legal_moves {
            self.board.make_move(mv);
            let score = -self.negamax(depth - 1, ply + 1, -beta, -alpha);
            self.board.undo_move();

            if score > best_score {
                best_score = score;
            }
            alpha = alpha.max(best_score);

            if alpha >= beta {
                break;
            }
        }

        best_score
    }

    pub fn find_best_move(&mut self, depth: u8) -> (Move, i32) {
        let mut possible_moves = self.board.generate_legal_moves();

        possible_moves.sort_unstable_by_key(|mv| std::cmp::Reverse(self.move_order_score(*mv)));

        let mut best_move = possible_moves[0];
        let mut alpha = i32::MIN + 1;
        let beta = i32::MAX;

        for mv in possible_moves {
            self.board.make_move(mv);

            let score = -self.negamax(depth - 1, 1, -beta, -alpha);

            self.board.undo_move();

            if score > alpha {
                alpha = score;
                best_move = mv;
            }
        }

        (best_move, alpha)
    }

    pub fn quiescence(&mut self, alpha: i32, beta: i32) -> i32 {
        Evaluator::new(self.board).static_eval()
    }

    fn mate_score(&self, ply: i32) -> i32 {
        MATE_SCORE - ply
    }

    fn move_order_score(&self, mv: Move) -> i32 {
        let captured = mv.captured_piece();

        if captured <= 5 {
            let victim = PieceKind::from_idx(captured as usize).value();
            let attacker = PieceKind::from_idx(mv.piece() as usize).value();

            return 10_000 + victim * 10 - attacker;
        }

        match MoveFlag::from_bits(mv.flags()) {
            MoveFlag::PromoQueen => 20_000,
            MoveFlag::PromoRook => 19_000,
            MoveFlag::PromoBishop => 18_000,
            MoveFlag::PromoKnight => 17_000,
            MoveFlag::EpCapture => 10_000 + 900,
            _ => 0,
        }
    }
}
