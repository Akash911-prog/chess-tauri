use std::sync::Mutex;

use crate::engine::board::Board;

pub struct Game {
    board: Mutex<Board>,
}

impl Game {
    pub fn new() -> Game {
        Game {
            board: Mutex::new(Board::new()),
        }
    }

    pub fn init(&mut self) {
        let move_gen = &mut self.board.lock().unwrap().move_gen;
        move_gen.generate_moves();
    }
}

impl Default for Game {
    fn default() -> Self {
        Game::new()
    }
}
