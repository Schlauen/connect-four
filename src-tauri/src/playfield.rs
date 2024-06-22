use array2d::Array2D;
use serde::{Serialize, Deserialize};
use crate::engine::{self, WIDTH};

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[repr(i8)]
pub enum CellState {
    Blank=0,
    P1=1,
    P2=-1,
}

#[derive(serde::Serialize, Clone)]
struct CellUpdateEvent {
    row: u8,
    col: u8,
    state: u8,
}

#[derive(serde::Serialize, Clone)]
pub struct UpdateEvent {
    state: u8,
    winner: Option<i8>,
    cell_updates: Option<CellUpdateEvent>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Cell {
    row:usize,
    col:usize,
    state: CellState,
}

impl Cell {
    fn set_state(&mut self, state:CellState) -> Result<Option<CellUpdateEvent>, String> {
        let changed = state != self.state;
        match self.state {
            CellState::P1 | CellState::P2 => {
                return Err("Cell is already set".into());
            }
            CellState::Blank => {
                self.state = state;
            }
        }
        
        Ok(match changed {
            true => Option::Some(CellUpdateEvent {
                row: self.row as u8,
                col: self.col as u8,
                state: self.state as u8,
            }),
            false => Option::None 
        })
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum GameState {
    Blank,
    Running,
    Finished,
}

pub struct Game {
    cells: Array2D<Cell>,
    state: GameState,
    col_heights: [usize; engine::WIDTH],
    current_player: CellState,
}

impl Game {
    pub fn new() -> Game {
        let mut counter = 0;
        let increment = || {
            let col = counter % WIDTH;
            let row = counter / WIDTH;
            counter += 1;
            Cell { row: row, col: col, state: CellState::Blank }
        };
        Game {
            cells: Array2D::filled_by_row_major(increment, engine::HEIGHT, engine::WIDTH),
            state: GameState::Blank,
            col_heights: [0; engine::WIDTH],
            current_player: CellState::P1
        }
    }

    fn map_values(&self) -> Array2D<i8> {
        let mut counter = 0;
        let increment = || {
            let col = counter % WIDTH;
            let row = counter / WIDTH;
            counter += 1;
            self.cells[(row, col)].state as i8
        };
        Array2D::filled_by_row_major(
            increment, 
            engine::HEIGHT, 
            engine::WIDTH
        )
    }

    pub fn play_col(&mut self, col:usize, player:CellState) -> Result<UpdateEvent, String> {
        match self.state {
            GameState::Blank => self.state = GameState::Running,
            GameState::Finished => return Err("Already solved".into()),
            GameState::Running => {}        
        };
        
        let row = self.col_heights[col];
        self.col_heights[col] = row + 1;

        let cell_event = self.cells[(row, col)].set_state(player)?;
        cell_event.map(|event| {
            let winner = engine::get_evaluation(
                self.map_values(), 
                self.col_heights,
                self.current_player as i8
            ).winner;
            
            UpdateEvent {
                state: self.state as u8,
                winner: winner,
                cell_updates: Option::Some(event)
            }
        }).ok_or("Cell not changed".into())
    }

    pub fn auto_play(&mut self, player:CellState) -> Result<UpdateEvent, String> {
        match self.state {
            GameState::Blank => self.state = GameState::Running,
            GameState::Finished => return Err("Already solved".into()),
            GameState::Running => {}        
        };
        
        let best_move = engine::get_best_move(
            Option::Some(self.map_values()),
            Option::Some(self.col_heights),
            player as i8
        )?;
        self.play_col(best_move, player)
    }
}