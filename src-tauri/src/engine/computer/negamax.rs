use rand::seq::SliceRandom;
use rand::thread_rng;
use std::{
    char::MAX,
    cmp::Reverse,
    time::{Duration, Instant},
};

use crate::engine::{
    board::Board,
    computer::{
        evaluator::Evaluator,
        tt::{Bound, TTEntry, TranspositionTable},
    },
    constants::{INF, MATE_SCORE, MATE_THRESHOLD, MAX_DEPTH, MAX_MOVES},
    movegen::{Move, MoveFlag},
    types::PieceKind,
};

const MAX_PLY: usize = 128;

use std::sync::OnceLock;

static LMR_TABLE: OnceLock<Vec<Vec<i32>>> = OnceLock::new();

fn init_lmr() -> Vec<Vec<i32>> {
    let mut table = vec![vec![0i32; MAX_MOVES]; MAX_DEPTH];

    for depth in 1..MAX_DEPTH {
        for move_count in 1..MAX_MOVES {
            if depth < 3 || move_count < 3 {
                continue;
            }

            let r = (depth as f64).ln() * (move_count as f64).ln() / 1.75;
            table[depth][move_count] = r as i32;
        }
    }
    table
}

pub struct Search<'a> {
    board: &'a mut Board,
    pub nodes_visited: u64,
    tt: TranspositionTable,
    move_buffers: Vec<Vec<Move>>,
    aborted: bool,
    pub depth: u8, // later: tt: TranspositionTable, killers: [[Move; 2]; MAX_DEPTH], stop_time: Instant, ...
}

impl<'a> Search<'a> {
    pub fn new(board: &'a mut Board) -> Self {
        let mut move_buffers = Vec::with_capacity(MAX_PLY);

        for _ in 0..MAX_PLY {
            move_buffers.push(Vec::with_capacity(64));
        }

        Search {
            board,
            nodes_visited: 0,
            tt: TranspositionTable::new(64),
            move_buffers,
            aborted: false,
            depth: 1,
        }
    }

    pub fn negamax(
        &mut self,
        depth: u8,
        ply: i32,
        mut alpha: i32,
        mut beta: i32,
        timer: &Instant,
        time_limit: &Duration,
        allow_null_move: bool,
    ) -> i32 {
        self.nodes_visited += 1;

        if self.nodes_visited % 3000 == 0 && timer.elapsed() >= *time_limit {
            self.aborted = true;
            return 0;
        }

        let original_alpha = alpha;
        let hash = self.board.zobrist_hash;

        let probe = self.tt.probe(hash);
        let tt_move = probe.and_then(|entry| entry.best_move);

        if let Some(entry) = probe {
            if entry.depth >= depth {
                let mut tt_score = entry.score;
                if tt_score >= MATE_THRESHOLD {
                    tt_score -= ply;
                } else if tt_score <= -MATE_THRESHOLD {
                    tt_score += ply;
                }

                match entry.bound {
                    Bound::Exact => return tt_score,
                    Bound::Lower => alpha = alpha.max(tt_score),
                    Bound::Upper => beta = beta.min(tt_score),
                }

                if alpha >= beta {
                    return tt_score;
                }
            }
        }

        let check_info = self.board.check_for_check();

        if (!check_info.is_check)
            & (depth >= 3)
            & (self.board.has_non_pawn_mat())
            & allow_null_move
            & (beta < MATE_THRESHOLD)
            & (beta > -MATE_THRESHOLD)
        {
            const NULL_MOVE_REDUCTION: u8 = 2;
            let old_en_passant = self.board.make_null_move();
            let score = -self.negamax(
                depth - 1 - NULL_MOVE_REDUCTION,
                ply + 1,
                -beta,
                -beta + 1,
                timer,
                time_limit,
                false,
            );
            self.board.undo_null_move(old_en_passant);

            // A subtree that hit the time limit returns a placeholder 0,
            // not a real evaluation - never trust it for a cutoff.
            if self.aborted {
                return 0;
            }

            if score >= beta {
                return beta;
            }
        }

        let mut legal_moves = std::mem::take(&mut self.move_buffers[ply as usize]);

        self.board.generate_legal_moves(&mut legal_moves);

        if legal_moves.is_empty() {
            let result = if check_info.is_check {
                -(self.mate_score(ply))
            } else {
                0
            };
            self.move_buffers[ply as usize] = legal_moves;
            return result;
        }

        legal_moves.sort_unstable_by_key(|mv| {
            if Some(*mv) == tt_move {
                Reverse(i32::MAX)
            } else {
                Reverse(self.move_order_score(*mv))
            }
        });

        if depth == 0 {
            let score = self.quiescence(alpha, beta);
            self.move_buffers[ply as usize] = legal_moves;
            return score;
        }

        let mut best_score = i32::MIN + 1;
        let mut best_move = None;
        let mut score;
        let mut i = 0;

        // Indexed rather than `for mv in &legal_moves` so the vector isn't
        // borrowed for the whole loop - we need to move it back into the
        // buffer pool on the early return below.
        while i < legal_moves.len() {
            let mv = legal_moves[i];
            self.board.make_move(mv);

            let mut reduction = 0;
            if (i >= 3) & (depth >= 3) & (mv.flags() == MoveFlag::Quiet.bits()) {
                reduction = self.get_lmr_reduction(depth.into(), i as i32);
                reduction = reduction.min((depth - 1) as i32);
            }

            if i == 0 {
                score = -self.negamax(
                    depth - 1 - reduction as u8,
                    ply + 1,
                    -beta,
                    -alpha,
                    timer,
                    time_limit,
                    true,
                );
            } else {
                score = -self.negamax(
                    depth - 1 - reduction as u8,
                    ply + 1,
                    -alpha - 1,
                    -alpha,
                    timer,
                    time_limit,
                    true,
                );

                if (reduction > 0) & (score > alpha) {
                    score =
                        -self.negamax(depth - 1, ply + 1, -beta, -alpha, timer, time_limit, true);
                }

                if (score > alpha) & (score < beta) {
                    score =
                        -self.negamax(depth - 1, ply + 1, -beta, -alpha, timer, time_limit, true);
                }
            }

            self.board.undo_move();

            // A subtree that hit the time limit returns a placeholder 0,
            // not a real evaluation - discard it instead of treating it as
            // this move's score, and don't cache it in the TT below.
            if self.aborted {
                self.move_buffers[ply as usize] = legal_moves;
                return 0;
            }

            if score > best_score {
                best_score = score;
                best_move = Some(mv);
            }
            alpha = alpha.max(best_score);

            if alpha >= beta {
                break;
            }

            i += 1;
        }

        let bound = if best_score <= original_alpha {
            Bound::Upper
        } else if best_score >= beta {
            Bound::Lower
        } else {
            Bound::Exact
        };

        let tt_score = if best_score >= MATE_THRESHOLD {
            best_score + ply
        } else if best_score <= -MATE_THRESHOLD {
            best_score - ply
        } else {
            best_score
        };

        self.tt.store(TTEntry {
            key: hash,
            depth,
            score: tt_score,
            bound,
            best_move,
        });

        self.move_buffers[ply as usize] = legal_moves;

        best_score
    }

    pub fn get_lmr_reduction(&self, depth: i32, move_count: i32) -> i32 {
        let depth_c = depth.min(MAX_DEPTH as i32 - 1);
        let move_count_c = move_count.min(MAX_MOVES as i32 - 1);

        if (depth_c as usize) < MAX_DEPTH && (move_count_c as usize) < MAX_MOVES {
            LMR_TABLE.get_or_init(init_lmr)[depth_c as usize][move_count_c as usize] as i32
        } else {
            // Out-of-bounds fallback: same formula, computed live
            Self::lmr_formula(depth_c, move_count_c) as i32
        }
    }

    #[inline]
    fn lmr_formula(depth: i32, move_count: i32) -> i32 {
        let depth = depth.max(1) as f64;
        let move_count = move_count.max(1) as f64;
        (0.75 + depth.ln() * move_count.ln() / 2.25) as i32
    }

    pub fn find_best_move(&mut self, alloted_time: u64) -> (Move, i32) {
        let time_limit = Duration::from_millis(alloted_time);
        let start = Instant::now();

        let mut possible_moves = Vec::with_capacity(64);
        self.board.generate_legal_moves(&mut possible_moves);

        let mut best_move = possible_moves[0];
        let mut best_score = 0;
        let mut depth = 1;

        loop {
            // Stop before starting a new depth if we're out of budget.
            if start.elapsed() >= time_limit {
                break;
            }

            let tt_move = self
                .tt
                .probe(self.board.zobrist_hash)
                .and_then(|entry| entry.best_move);

            // Search the TT move first, then other moves by move-ordering score.
            possible_moves.sort_unstable_by_key(|mv| {
                if Some(*mv) == tt_move {
                    Reverse(i32::MAX)
                } else {
                    Reverse(self.move_order_score(*mv))
                }
            });

            let mut alpha = -INF;
            let beta = INF;

            let mut scored_moves: Vec<(Move, i32)> = Vec::with_capacity(possible_moves.len());

            for mv in &possible_moves {
                self.board.make_move(*mv);

                let score = -self.negamax(depth - 1, 1, -beta, -alpha, &start, &time_limit, true);

                self.board.undo_move();

                scored_moves.push((*mv, score));

                if score > alpha {
                    alpha = score;
                }

                // Bail mid-depth if we've blown the budget.
                // This depth's results are partial and unreliable.
                if start.elapsed() >= time_limit {
                    self.aborted = true;
                    break;
                }
            }

            if self.aborted {
                break;
            }

            // Pick the actual highest-scoring move.
            let (depth_best_move, depth_best_score) = scored_moves
                .iter()
                .max_by_key(|(_, score)| *score)
                .copied()
                .unwrap();

            best_move = depth_best_move;
            best_score = depth_best_score;

            // Reorder for the next iteration so the best move from this
            // depth gets searched first.
            if let Some(pos) = possible_moves.iter().position(|mv| *mv == best_move) {
                possible_moves.swap(0, pos);
            }

            depth += 1;
        }

        self.depth = depth;

        (best_move, best_score)
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
