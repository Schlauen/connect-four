// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod engine;
mod minimax;
mod playfield;

use std::sync::Mutex;
use playfield::Game;
use tauri::Window;

// Mutex for interior mutability
struct PlayfieldState {
    playfield: Mutex<Game>,
    human_player: playfield::CellState,
    computer_player: playfield::CellState,
}

#[tauri::command]
fn play_col(
    state:tauri::State<'_, PlayfieldState>,
    window: Window,
    col:usize
) -> Result<(), String> {
    let mut playfield = state.playfield.lock().unwrap();
    playfield.play_col(col, state.human_player).map(|_| todo!("handle update events"))?;
    playfield.auto_play(state.computer_player).map(|_| todo!("handle update events"))
}

fn main() {
    tauri::Builder::default()
        .manage(PlayfieldState {
            playfield: Mutex::new(Game::new()),
            human_player: playfield::CellState::P1,
            computer_player: playfield::CellState::P2,
        })
        .invoke_handler(tauri::generate_handler![play_col])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
