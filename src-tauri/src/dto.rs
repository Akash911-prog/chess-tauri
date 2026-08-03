use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Listener, Manager};

use crate::engine::game::Game;

#[derive(Debug, Deserialize)]
pub struct MoveInfo {
    pub from: String,
    pub to: String,
    pub promotion: Option<PromotionPiece>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PromotionPiece {
    Q,
    R,
    B,
    N,
}

#[derive(Debug, Deserialize)]
pub struct GetLegalMovesParams {
    pub piece_type: String,
    pub square: String,
    pub color: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Legality {
    Legal = 0,
    Illegal = 1,
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", content = "data")]
pub enum CommandError {
    InvalidSquareIndex { square: u16 }, // out of 0..64 range
    EmptySquare { square: u8 },         // no piece to move/query at all
    GameAlreadyOver,                    // checkmate/stalemate/draw already reached
}

// ----------------------------
//          COMMANDS
// ----------------------------

// remember to call `.manage(MyState::default())`
#[tauri::command]
pub fn get_legal_moves(move_info: GetLegalMovesParams) -> Result<(), CommandError> {
    println!("{:?}", move_info);
    Ok(())
}

#[tauri::command]
pub fn update(app: AppHandle, move_info: MoveInfo) -> Result<Legality, CommandError> {
    println!("{:?}", move_info);
    let game_state = app.state::<Mutex<Game>>();
    let game = game_state.lock().unwrap();

    let mut board = game.board.lock().unwrap();
    let result = board.parse_react_move(move_info)?;
    Ok(result)
}

#[tauri::command]
pub fn show_window(app: AppHandle) -> Result<(), CommandError> {
    app.get_webview_window("main").unwrap().show().unwrap();
    return Ok(());
}

// ----------------------------
//          EVENTS
// ----------------------------
pub fn app_start_listener(app: &AppHandle) {
    let app_clone = app.clone();

    app.listen("init:start", move |e| {
        let game_state = app_clone.state::<Mutex<Game>>();
        let mut game = game_state.lock().unwrap();
        game.init();
        let _ = app_clone.emit("init:end", ());
        println!("init:end");
    });
}

pub fn init(handle: &AppHandle) {
    app_start_listener(handle);
}
