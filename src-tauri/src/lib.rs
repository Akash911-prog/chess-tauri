mod dto;
mod engine;
#[cfg(test)]
mod tests;

use crate::engine::game::Game;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle();

            dto::init(handle);

            Ok(())
        })
        .manage(Mutex::new(Game::new()))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            dto::get_legal_moves,
            dto::update,
            dto::show_window,
            dto::undo_move,
            dto::restart,
            dto::get_move
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
