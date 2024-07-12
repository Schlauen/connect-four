// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod engine;
mod minimax;
mod playfield;

use std::sync::Mutex;
use playfield::{Game, GameState};
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
    let game_state = playfield.play_col(col, state.human_player, Some(&window))?;

    match game_state {
        GameState::Finished => Ok(()),
        GameState::Blank | GameState::Calculating => Err("Cannot be blank or calculating".into()),
        GameState::Running => playfield.auto_play(state.computer_player, Some(&window))
    }
}

#[tauri::command]
fn new_game(
    state:tauri::State<'_, PlayfieldState>,
    window: Window,
    level:u8,
    starting_player:i8,
) -> Result<(), String> {
    let mut playfield = state.playfield.lock().unwrap();
    playfield.reset(level, Some(&window))?;

    if starting_player == state.computer_player as i8 {
        return playfield.auto_play(state.computer_player, Some(&window))
    }
    Result::Ok(())
}

fn main() {
    tauri::Builder::default()
        .manage(PlayfieldState {
            playfield: Mutex::new(Game::new(8)),
            human_player: playfield::CellState::P1,
            computer_player: playfield::CellState::P2,
        })
        .invoke_handler(tauri::generate_handler![play_col, new_game])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
