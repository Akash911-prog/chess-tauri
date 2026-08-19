use std::{sync::Mutex, time::Instant};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Listener, Manager};
use tracing::{debug, info, instrument};

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
#[instrument]
pub fn get_legal_moves(move_info: GetLegalMovesParams) -> Result<(), CommandError> {
    debug!(?move_info, "get_legal_moves received");
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

    square_changes.extend(move_changes);

    board.player_turn ^= 1;
    let game_state = board.get_game_state();
    debug!(?game_state, "game state after move");
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
#[instrument(skip(app))]
pub fn get_move(app: AppHandle) -> Result<Legality, CommandError> {
    info!("AI move requested");

    let game_state = app.state::<Mutex<Game>>();
    let game = game_state.lock().unwrap();

    let mut board = game.board.lock().unwrap();
    let result: (Move, i32);
    let search_started = Instant::now();
    let (nodes_visited, depth_reached);
    {
        let mut evaluator = Search::new(&mut (*board));
        result = evaluator.find_best_move(1000);
        nodes_visited = evaluator.nodes_visited;
        depth_reached = evaluator.depth;
    }
    let elapsed = search_started.elapsed();

    info!(
        from = %board.index_to_notation(result.0.from()),
        to = %board.index_to_notation(result.0.to()),
        score = result.1,
        nodes_visited,
        depth_reached,
        elapsed_ms = elapsed.as_millis() as u64,
        "AI move chosen"
    );

    board.make_move(result.0);

    let response = build_response(result.0, &mut board);
    if response.condition != GameState::InProgress {
        let condition = &response.condition;
        let winner = &response.winner;
        info!(?condition, ?winner, "game ended after AI move");
    }
    let result = Legality::Legal(response);

    Ok(result)
}

#[tauri::command]
#[instrument(skip(app))]
pub fn update(app: AppHandle, move_info: MoveInfo) -> Result<Legality, CommandError> {
    info!(
        from = %move_info.from,
        to = %move_info.to,
        promotion = ?move_info.promotion,
        "move received from frontend"
    );

    let game_state = app.state::<Mutex<Game>>();
    let game = game_state.lock().unwrap();

    let mut board = game.board.lock().unwrap();
    let result = board.parse_react_move(move_info);

    match &result {
        Ok(Legality::Legal(response)) => {
            debug!(?response.condition, "move applied");
        }
        Ok(Legality::Illegal) => {
            debug!("move rejected as illegal");
        }
        Err(err) => {
            debug!(?err, "move rejected with error");
        }
    }

    result
}

#[tauri::command]
pub fn show_window(app: AppHandle) -> Result<(), CommandError> {
    app.get_webview_window("main").unwrap().show().unwrap();
    return Ok(());
}

#[tauri::command]
#[instrument(skip(app))]
pub fn undo_move(app: AppHandle) -> Result<(), CommandError> {
    info!("undo_move requested");
    let game_state = app.state::<Mutex<Game>>();
    let game = game_state.lock().unwrap();

    let mut board = game.board.lock().unwrap();
    board.undo_move();
    Ok(())
}

#[tauri::command]
#[instrument(skip(app))]
pub fn restart(app: AppHandle) -> Result<(), CommandError> {
    info!("restart requested");
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

    app.listen("init:start", move |_e| {
        let game_state = app_clone.state::<Mutex<Game>>();
        let mut game = game_state.lock().unwrap();
        game.init();
        let _ = app_clone.emit("init:end", ());
        info!("game initialized");
    });
}

pub fn init(handle: &AppHandle) {
    app_start_listener(handle);
}
