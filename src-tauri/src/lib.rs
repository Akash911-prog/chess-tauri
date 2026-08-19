mod dto;
mod engine;
#[cfg(test)]
mod tests;

use tauri::Manager;

use crate::engine::game::Game;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Rotates daily under the app's data dir; falls back to the working
    // directory in dev. Boundary-only logging (frontend I/O, AI move
    // results, game lifecycle) — never inside negamax/quiescence, since
    // that's per-node and would tank search speed.
    let log_dir = dirs::data_local_dir()
        .map(|d| d.join("chess-tauri").join("logs"))
        .unwrap_or_else(|| std::path::PathBuf::from("logs"));
    let file_appender = tracing_appender::rolling::daily(log_dir, "chess-tauri.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tauri::Builder::default()
        .setup(move |app| {
            // Keep the guard alive for the app's lifetime by handing it to
            // Tauri's managed state — dropping it silently stops flushing.
            app.manage(_guard);

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
