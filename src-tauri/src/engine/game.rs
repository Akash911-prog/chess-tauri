use std::sync::Mutex;

use crate::engine::board::Board;

pub struct Game {
    pub board: Mutex<Board>,
}

impl Game {
    pub fn new() -> Game {
        Game {
            board: Mutex::new(Board::new()),
        }
    }

    pub fn init(&mut self) {
        {
            let mut board = self.board.lock().unwrap();
            board.move_gen.generate_moves();
        } // guard dropped here

        let mut board = self.board.lock().unwrap();
        board.init();
    } // guard dropped here

    pub fn restart(&mut self) {
        let mut board = self.board.lock().unwrap();
        board.reset();
    }
}

impl Default for Game {
    fn default() -> Self {
        Game::new()
    }
}
