use std::{borrow::BorrowMut, collections::{VecDeque}};

use array2d::Array2D;
use serde::{Serialize, Deserialize};
use tauri::Window;
use crate::engine::{self, Eval, EvaluationResult, TOTAL_FIELDS, WIDTH};

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[repr(i8)]
pub enum CellState {
    Blank=0,
    P1=1,
    P2=-1,
}

#[derive(serde::Serialize, Clone)]
pub struct CellUpdateEvent {
    pub row: u8,
    pub col: u8,
    state: i8,
    winning: bool,
}

#[derive(serde::Serialize, Clone)]
pub struct UpdateEvent {
    state: i8,
    winner: Option<i8>,
    balance_of_power:Option<f32>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Cell {
    row:usize,
    col:usize,
    state: CellState,
    winning:bool,
}

impl Cell {
    fn emit_update(&self, window:Option<&Window>) {
        window.map(|w| {
            w.emit(
                &format!("updateCell-{}-{}", self.row, self.col), 
                CellUpdateEvent {
                    row: self.row as u8,
                    col: self.col as u8,
                    state: self.state as i8,
                    winning: self.winning
                }
            ).unwrap()
        });
    }

    fn reset(&mut self, window:Option<&Window>) {
        self.state = CellState::Blank;
        self.winning = false;
        self.emit_update(window);
    }

    fn set_state(&mut self, state:CellState, window:Option<&Window>) -> Result<bool, String> {
        if state == self.state {
            return Ok(false);
        }

        let result = match self.state {
            CellState::P1 => match state {
                CellState::Blank => {
                    self.state = state;
                    Result::<bool, String>::Ok(true)
                },
                CellState::P2 => Err("Cell is already set".into()),
                CellState::P1 => Ok(false)
            }
            CellState::P2 => match state {
                CellState::Blank => {
                    self.state = state;
                    Ok(true)
                },
                CellState::P1 => Err("Cell is already set".into()),
                CellState::P2 => Ok(false)
            }
            CellState::Blank =>  match state {
                CellState::Blank => Ok(false),
                CellState::P1 | CellState::P2 => {
                    self.state = state;
                    Ok(true)
                },
            }
        }?;
        
        self.emit_update(window);
        Ok(result)
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum GameState {
    Blank,
    Running,
    Finished,
    Calculating,
}

pub struct Game {
    cells: Array2D<Cell>,
    state: GameState,
    col_heights: [usize; engine::WIDTH],
    current_player: CellState,
    level:u8,
    move_history: VecDeque<usize>,
}

impl Game {
    pub fn new(level:u8) -> Game {
        let mut counter = 0;
        let increment = || {
            let col = counter % WIDTH;
            let row = counter / WIDTH;
            counter += 1;
            Cell { row: row, col: col, state: CellState::Blank, winning: false }
        };
        Game {
            cells: Array2D::filled_by_row_major(increment, engine::HEIGHT, engine::WIDTH),
            state: GameState::Blank,
            col_heights: [0; engine::WIDTH],
            current_player: CellState::P1,
            level: level,
            move_history: VecDeque::with_capacity(TOTAL_FIELDS),
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

    fn emit_update(&self, event:UpdateEvent, window:Option<&Window>) {
        window.map(|w|w.emit("updateGame", event).unwrap());
    }

    fn evaluate(&self) -> EvaluationResult {
        self.move_history.back()
                .map(|col| {
                    engine::evaluate_action(
                        (self.map_values(), self.current_player as i8),
                        *col
                    )
                })
                .unwrap_or(EvaluationResult {
                    eval: Eval {
                        score: 0.,
                        finished: false,
                        winner: None
                    },
                    winning_cells: Option::None
                })
    }

    pub fn play_col(&mut self, col:usize, player:CellState, window:Option<&Window>) -> Result<(), String> {
         match self.state {
            GameState::Blank => {
                self.state = GameState::Running;
                Ok::<(),String>(())
            },
            GameState::Finished => Err("Already solved".into()),
            GameState::Calculating => Err("calculating".into()),
            GameState::Running => Ok(())
        }?;
        self.current_player = player;        
        let row = self.col_heights[col];
        self.col_heights[col] = row + 1;
        self.move_history.push_back(col);

        match self.cells[(row, col)].set_state(player, window)? {
            true => {
                let result = self.evaluate();
                
                if result.eval.finished {
                    self.state = GameState::Finished;
                }
                
                self.emit_update(
                    UpdateEvent {
                        state: self.state as i8,
                        winner: result.eval.winner,
                        balance_of_power: Option::Some(result.eval.score)
                    }, 
                    window
                );

                result.winning_cells.map(|winning_cells| {
                    for coords in winning_cells {
                        let cell = self.cells[coords].borrow_mut();
                        cell.winning = true;
                        cell.emit_update(window);
                    }
                });

                Ok(())
            }
            false => {
                Err("Cell not changed".into())
            }
        }
    }

    fn get_best_move(&self, player:CellState) -> Result<usize, String> {
        engine::get_best_move(
            Option::Some(self.map_values()),
            player as i8,
            self.level
        )
    }

    pub fn auto_play(&mut self, player:CellState, window:Option<&Window>) -> Result<(), String> {
        match self.state {
            GameState::Blank => self.state = GameState::Running,
            GameState::Finished => return Err("Already solved".into()),
            GameState::Calculating => return Err("calculating".into()),
            GameState::Running => {}        
        };

        self.emit_update(
            UpdateEvent {
                state: GameState::Calculating as i8,
                winner: None,
                balance_of_power: None,
            },
            window
        );
        
        
        let best_move = self.get_best_move(player)?;
        self.play_col(best_move, player, window)
    }

    pub fn reset(&mut self, level:u8, window:Option<&Window>) -> Result<(), String> {
        for h in self.col_heights.iter_mut() {
            *h = 0;
        }

        for (row, col) in (0..engine::HEIGHT).flat_map(|r| (0..engine::WIDTH).map(move |c| (r,c))) {
            let cell = self.cells[(row, col)].borrow_mut();
            cell.reset(window);
        }

        self.state = GameState::Blank;
        self.current_player = CellState::P1;
        self.level = level;

        self.emit_update(
            UpdateEvent {
                state: GameState::Blank as i8,
                winner: Option::None,
                balance_of_power: Some(0.)
            }, 
            window
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enum() {
        assert_eq!(CellState::P2 as i8, -1);
    }

    #[test]
    fn test_play() {
        let mut g = Game::new(8);
        let (x,o) = (CellState::P1, CellState::P2);
        g.play_col(3, x, None).unwrap();
        g.play_col(2, o, None).unwrap();
        g.play_col(4, x, None).unwrap();
        g.play_col(3, o, None).unwrap();
        g.play_col(6, x, None).unwrap();
        
        assert_eq!(g.get_best_move(o).unwrap(), 5);
        assert_eq!(g.get_best_move(x).unwrap(), 5);

        g.play_col(5, o, None).unwrap();
        g.play_col(5, x, None).unwrap();
        g.play_col(2, o, None).unwrap();
        g.play_col(5, x, None).unwrap();
        g.play_col(5, o, None).unwrap();
        g.play_col(2, x, None).unwrap();
        g.play_col(3, o, None).unwrap();
        g.play_col(0, x, None).unwrap();

        assert_eq!(g.get_best_move(o).unwrap(), 4);
        assert_eq!(g.get_best_move(x).unwrap(), 4);        
    }

    #[test]
    fn test_play_2() {
        let mut g = Game::new(8);
        let (x,o) = (CellState::P1, CellState::P2);
        g.play_col(2, x, None).unwrap();
        g.play_col(6, o, None).unwrap();
        g.play_col(3, x, None).unwrap();
        g.play_col(6, o, None).unwrap();
        g.play_col(6, x, None).unwrap();
        g.play_col(5, o, None).unwrap();
        g.play_col(1, x, None).unwrap();
        g.play_col(0, o, None).unwrap();

        assert_eq!(g.get_best_move(x).unwrap(), 4);
        assert_eq!(g.get_best_move(o).unwrap(), 4);

        g.play_col(4, x, None).unwrap();

        let result = g.evaluate();
        assert_eq!(result.eval.winner.unwrap(), x as i8);
        
        //g.play_col(4, player, window)
        //g.play_col(1, x, None).unwrap();    
    }

    #[test]
    fn test_play_3() {
        let mut g = Game::new(8);
        let (x,o) = (CellState::P1, CellState::P2);
        g.play_col(1, x, None).unwrap();
        g.play_col(6, o, None).unwrap();
        g.play_col(2, x, None).unwrap();

        assert_eq!(g.get_best_move(o).unwrap(), 0);

        g.play_col(6, o, None).unwrap();

        assert_eq!(g.get_best_move(x).unwrap(), 3);        
    }
}