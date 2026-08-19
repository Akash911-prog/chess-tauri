use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Listener, Manager};

use crate::engine::{
    board::Board,
    computer::negamax::Search,
    game::Game,
    movegen::{Move, MoveFlag},
    types::{Color, PieceKind},
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveInfo {
    pub from: String,
    pub to: String,
    pub promotion: Option<PromotionPiece>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PromotionPiece {
    Knight,
    Bishop,
    Rook,
    Queen,
}

#[derive(Debug, Deserialize)]
pub struct GetLegalMovesParams {
    pub piece_type: String,
    pub square: String,
    pub color: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "kind", content = "data", rename_all = "lowercase")]
pub enum Legality {
    Legal(Response),
    Illegal,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MoveType {
    Normal,
    Castling,
    Promotion,
    EnPassant,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PieceKindDto {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl From<PieceKindDto> for crate::engine::types::PieceKind {
    fn from(value: PieceKindDto) -> Self {
        match value {
            PieceKindDto::Pawn => crate::engine::types::PieceKind::Pawn,
            PieceKindDto::Knight => crate::engine::types::PieceKind::Knight,
            PieceKindDto::Bishop => crate::engine::types::PieceKind::Bishop,
            PieceKindDto::Rook => crate::engine::types::PieceKind::Rook,
            PieceKindDto::Queen => crate::engine::types::PieceKind::Queen,
            PieceKindDto::King => crate::engine::types::PieceKind::King,
        }
    }
}

impl From<crate::engine::types::PieceKind> for PieceKindDto {
    fn from(value: crate::engine::types::PieceKind) -> Self {
        match value {
            crate::engine::types::PieceKind::Pawn => PieceKindDto::Pawn,
            crate::engine::types::PieceKind::Knight => PieceKindDto::Knight,
            crate::engine::types::PieceKind::Bishop => PieceKindDto::Bishop,
            crate::engine::types::PieceKind::Rook => PieceKindDto::Rook,
            crate::engine::types::PieceKind::Queen => PieceKindDto::Queen,
            crate::engine::types::PieceKind::King => PieceKindDto::King,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PieceInfo {
    pub kind: PieceKindDto,
    pub color: Color,
}

impl PieceInfo {
    pub fn new(kind: PieceKindDto, color: Color) -> Self {
        PieceInfo { kind, color }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SquareChange {
    pub square: String,
    pub piece: Option<PieceInfo>, // None = now empty
}

impl SquareChange {
    pub fn new(square: String, piece: Option<PieceInfo>) -> Self {
        SquareChange { square, piece }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub move_type: MoveType,
    pub changes: Vec<SquareChange>,
    pub condition: GameState,
    pub winner: Option<Color>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GameState {
    InProgress,
    Checkmate,
    Stalemate,
    Draw,
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

fn build_response(mv: Move, board: &mut Board) -> Response {
    let from = mv.from();
    let to = mv.to();

    board.player_turn ^= 1;

    let mut square_changes: Vec<SquareChange> = vec![];

    let mut promotion_kind: Option<PieceKindDto> = None;

    match MoveFlag::from_bits(mv.flags()) {
        MoveFlag::EpCapture => {
            let captured_idx = if board.player_turn == 0 {
                to - 8
            } else {
                to + 8
            };
            square_changes.push(SquareChange::new(
                board.index_to_notation(captured_idx),
                None,
            ));
        }
        MoveFlag::KingCastle => {
            let rook_from = if board.player_turn == 0 { 7 } else { 63 };
            let rook_to = if board.player_turn == 0 { 5 } else { 61 };
            square_changes.push(SquareChange::new(
                board.index_to_notation(rook_to),
                Some(PieceInfo::new(
                    PieceKindDto::Rook,
                    Color::from(board.player_turn),
                )),
            ));
            square_changes.push(SquareChange::new(board.index_to_notation(rook_from), None));
        }
        MoveFlag::QueenCastle => {
            let rook_from = if board.player_turn == 0 { 0 } else { 56 };
            let rook_to = if board.player_turn == 0 { 3 } else { 59 };
            square_changes.push(SquareChange::new(
                board.index_to_notation(rook_to),
                Some(PieceInfo::new(
                    PieceKindDto::Rook,
                    Color::from(board.player_turn),
                )),
            ));
            square_changes.push(SquareChange::new(board.index_to_notation(rook_from), None));
        }
        MoveFlag::PromoBishop => {
            promotion_kind = Some(PieceKindDto::Bishop);
        }
        MoveFlag::PromoKnight => {
            promotion_kind = Some(PieceKindDto::Knight);
        }
        MoveFlag::PromoQueen => {
            promotion_kind = Some(PieceKindDto::Queen);
        }
        MoveFlag::PromoRook => {
            promotion_kind = Some(PieceKindDto::Rook);
        }
        _ => {}
    }

    let move_changes = board.build_move_changes(
        from,
        to,
        PieceKind::from_idx(mv.piece() as usize),
        promotion_kind,
    );

    println!("color: {}", board.player_turn);

    square_changes.extend(move_changes);

    board.player_turn ^= 1;
    let game_state = board.get_game_state();
    let mut winner: Option<Color> = None;
    if game_state == GameState::Checkmate {
        winner = Some(Color::from(board.player_turn ^ 1));
    }

    Response {
        move_type: MoveType::Normal,
        changes: square_changes,
        condition: game_state,
        winner: winner,
    }
}

#[tauri::command]
pub fn get_move(app: AppHandle) -> Result<Legality, CommandError> {
    println!("get_move");
    let game_state = app.state::<Mutex<Game>>();
    let game = game_state.lock().unwrap();

    let mut board = game.board.lock().unwrap();
    let result: (Move, i32);
    {
        let mut evaluator = Search::new(&mut (*board));
        result = evaluator.find_best_move(1000);
        println!("nodes visited: {}", evaluator.nodes_visited);
        println!("depth: {}", evaluator.depth);
    }
    board.make_move(result.0);

    let response = build_response(result.0, &mut board);
    let result = Legality::Legal(response);

    Ok(result)
}

#[tauri::command]
pub fn update(app: AppHandle, move_info: MoveInfo) -> Result<Legality, CommandError> {
    println!("update");
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

#[tauri::command]
pub fn undo_move(app: AppHandle) -> Result<(), CommandError> {
    let game_state = app.state::<Mutex<Game>>();
    let game = game_state.lock().unwrap();

    let mut board = game.board.lock().unwrap();
    board.undo_move();
    Ok(())
}

#[tauri::command]
pub fn restart(app: AppHandle) -> Result<(), CommandError> {
    println!("restart");
    let game_state = app.state::<Mutex<Game>>();
    let mut game = game_state.lock().unwrap();
    game.restart();
    Ok(())
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
