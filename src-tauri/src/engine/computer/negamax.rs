use rand::seq::SliceRandom;
use rand::thread_rng;
use std::{
    cmp::Reverse,
    time::{Duration, Instant},
};

use crate::engine::{
    board::Board,
    computer::{
        evaluator::Evaluator,
        tt::{Bound, TTEntry, TranspositionTable},
    },
    constants::MATE_SCORE,
    movegen::{Move, MoveFlag},
    types::PieceKind,
};

const MAX_PLY: usize = 128;

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
                match entry.bound {
                    Bound::Exact => return entry.score,
                    Bound::Lower => alpha = alpha.max(entry.score),
                    Bound::Upper => beta = beta.min(entry.score),
                }

                if alpha >= beta {
                    return entry.score;
                }
            }
        }

        let mut legal_moves = std::mem::take(&mut self.move_buffers[ply as usize]);

        self.board.generate_legal_moves(&mut legal_moves);

        legal_moves.sort_unstable_by_key(|mv| {
            if Some(*mv) == tt_move {
                Reverse(i32::MAX)
            } else {
                Reverse(self.move_order_score(*mv))
            }
        });

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
        let mut best_move = None;

        for mv in &legal_moves {
            self.board.make_move(*mv);
            let score = -self.negamax(depth - 1, ply + 1, -beta, -alpha, timer, time_limit);
            self.board.undo_move();

            if score > best_score {
                best_score = score;
                best_move = Some(*mv);
            }
            alpha = alpha.max(best_score);

            if alpha >= beta {
                break;
            }
        }

        let bound = if best_score <= original_alpha {
            Bound::Upper
        } else if best_score >= beta {
            Bound::Lower
        } else {
            Bound::Exact
        };

        self.tt.store(TTEntry {
            key: hash,
            depth,
            score: best_score,
            bound,
            best_move,
        });

        best_score
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
            // stop before starting a new depth if we're out of budget
            if start.elapsed() >= time_limit {
                break;
            }

            let tt_move = self
                .tt
                .probe(self.board.zobrist_hash)
                .and_then(|entry| entry.best_move);

            possible_moves.sort_unstable_by_key(|mv| {
                if Some(*mv) == tt_move {
                    Reverse(i32::MAX)
                } else {
                    Reverse(self.move_order_score(*mv))
                }
            });

            let mut alpha = i32::MIN + 1;
            let beta = i32::MAX;
            let mut scored_moves: Vec<(Move, i32)> = Vec::with_capacity(possible_moves.len());

            for mv in &possible_moves {
                self.board.make_move(*mv);
                let score = -self.negamax(depth - 1, 1, -beta, -alpha, &start, &time_limit);
                self.board.undo_move();
                scored_moves.push((*mv, score));
                if score > alpha {
                    alpha = score;
                }

                // bail mid-depth if we've blown the budget; this depth's
                // results are partial and unreliable, so discard them
                if start.elapsed() >= time_limit {
                    self.aborted = true;
                    break;
                }
            }

            if self.aborted {
                break;
            }

            let depth_best_score = scored_moves.iter().map(|(_, s)| *s).max().unwrap();
            const EPSILON: i32 = 3; // centipawns
            let candidates: Vec<Move> = scored_moves
                .iter()
                .filter(|(_, s)| depth_best_score - s <= EPSILON)
                .map(|(mv, _)| *mv)
                .collect();

            best_move = *candidates.choose(&mut thread_rng()).unwrap();
            best_score = depth_best_score;

            // reorder for next iteration so the best move from this depth
            // gets searched first (helps alpha-beta cutoffs next pass)
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
