use std::{ops::Deref, sync::Mutex};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Listener, Manager};

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

// ----------------------------
//          COMMANDS
// ----------------------------

// remember to call `.manage(MyState::default())`
#[tauri::command]
pub fn get_legal_moves(move_info: GetLegalMovesParams) -> Result<(), String> {
    println!("{:?}", move_info);
    Ok(())
}

#[tauri::command]
pub fn update(move_info: MoveInfo) -> Result<(), String> {
    println!("{:?}", move_info);
    Ok(())
}

// ----------------------------
//          EVENTS
// ----------------------------
pub fn app_start_listener(app: &AppHandle) {
    let app_clone = app.clone();

    app.listen("app_start", move |e| {
        println!("app_start: {}", e.payload());
        let game_state = app_clone.state::<Mutex<Game>>();
        let mut game = game_state.lock().unwrap();
        game.init();
    });
}

pub fn init(handle: &AppHandle) {
    app_start_listener(handle);
}
