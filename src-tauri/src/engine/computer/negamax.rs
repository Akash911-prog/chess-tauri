use crate::engine::{
    board::Board, computer::evaluator::Evaluator, constants::MATE_SCORE, movegen::Move,
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

        let legal_moves = self.board.generate_legal_moves();
        println!("legal moves: {}", legal_moves.len());

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
        // root-level loop, calls self.negamax per candidate move
        let possible_moves = self.board.generate_legal_moves();

        let mut best_move = possible_moves[0];
        let mut best_score = i32::MIN + 1;

        for mv in possible_moves {
            self.board.make_move(mv);
            let score = -self.negamax(depth - 1, 1, i32::MIN + 1, i32::MAX);
            self.board.undo_move();

            if score > best_score {
                best_score = score;
                best_move = mv;
            }
        }

        (best_move, best_score)
    }

    pub fn quiescence(&mut self, alpha: i32, beta: i32) -> i32 {
        Evaluator::new(self.board).static_eval()
    }

    fn mate_score(&self, ply: i32) -> i32 {
        MATE_SCORE - ply
    }

    pub fn negate(&self, score: i32, ply: i32) -> i32 {
        let mate_score = self.mate_score(ply);
        if score == mate_score {
            return -(self.mate_score(ply + 1));
        };

        -score
    }
}
