use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MoveInfo {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Deserialize)]
pub struct GetLegalMovesParams {
    pub piece_type: String,
    pub square: String,
    pub color: String,
}

// remember to call `.manage(MyState::default())`
#[tauri::command]
pub fn get_legal_moves(move_info: GetLegalMovesParams) -> Result<(), String> {
    println!("{:?}", move_info);
    Ok(())
}
