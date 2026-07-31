mod dto;
mod engine;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![dto::get_legal_moves])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
