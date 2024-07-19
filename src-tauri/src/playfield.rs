use std::{borrow::BorrowMut, collections::VecDeque};

use array2d::Array2D;
use serde::{Serialize, Deserialize};
use tauri::Window;
use crate::engine::{self, ActionEvaluation, Eval, HEIGHT, TOTAL_FIELDS, WIDTH};

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[repr(i8)]
pub enum CellState {
    Blank=0,
    P1=1,
    P2=-1,
}

#[derive(serde::Serialize, Clone)]
pub enum Update {
    Cell {
        row: u8,
        col: u8,
        state: i8,
        winning: bool,
    },
    State {
        state: i8,
        winner: Option<i8>,
    },
    Balance {
        value: f32,
    }
} 

#[derive(Serialize, Deserialize, Clone)]
pub struct Cell {
    row:usize,
    col:usize,
    state: CellState,
    winning:bool,
}

fn emit_update(event:Update, window:&Window) -> Result<(), String> {
    let s = match event {
        Update::Balance { value: _ } => "updateBalance".to_owned(),
        Update::Cell { row, col, state: _, winning: _ } => format!("updateCell-{}-{}", row, col),
        Update::State { state: _, winner:_ } => "updateState".to_owned()
    };
    window.emit(&s, event).map_err(|e| e.to_string())
}

impl Cell {
    fn emit_update(&self, window:Option<&Window>) {
        window.map(|w| emit_update( 
            Update::Cell { 
                row: self.row as u8,
                col: self.col as u8,
                state: self.state as i8,
                winning: self.winning 
            },
            w
        ));
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

    fn evaluate(&self) -> ActionEvaluation {
        self.move_history.back()
                .map(|col| engine::evaluate_action(Some(self.map_values()), self.current_player as i8,*col))
                .unwrap_or(ActionEvaluation {
                    eval: Eval {
                        score: 0.,
                        finished: false,
                        winner: None
                    },
                    winning_cells: Option::None
                })
    }

    pub fn play_col(&mut self, col:usize, player:CellState, window:Option<&Window>) -> Result<GameState, String> {
        // println!("{:?}", col);
        match self.state {
            GameState::Blank => {
                self.state = GameState::Running;
                Ok::<(),String>(())
            },
            GameState::Finished => Err("Already finished".into()),
            GameState::Calculating => Err("calculating".into()),
            GameState::Running => Ok(())
        }?;
        self.current_player = player;        
        let row = self.col_heights[col];

        if row >= HEIGHT {
            return Err("column already full".into());
        }

        self.col_heights[col] = row + 1;
        self.move_history.push_back(col);

        match self.cells[(row, col)].set_state(player, window)? {
            true => {
                let result = self.evaluate();
                
                if result.eval.finished {
                    self.state = GameState::Finished;
                }
                
                window.map(|w| emit_update(Update::State { 
                    state: self.state as i8,
                    winner: result.eval.winner
                }, w));

                result.winning_cells.map(|winning_cells| {
                    for coords in winning_cells {
                        let cell = self.cells[coords].borrow_mut();
                        cell.winning = true;
                        cell.emit_update(window);
                    }
                });

                Ok(self.state)
            }
            false => {
                Err("Cell not changed".into())
            }
        }
    }

    pub fn auto_play(&mut self, player:CellState, window:Option<&Window>) -> Result<(), String> {
        match self.state {
            GameState::Blank => self.state = GameState::Running,
            GameState::Finished => return Err("Already solved".into()),
            GameState::Calculating => return Err("calculating".into()),
            GameState::Running => {}        
        };

        window.map(|w| emit_update(Update::State { 
            state: GameState::Calculating as i8,
            winner: None
        }, w));
        
        
        let res = engine::evaluate_state(Some(self.map_values()), player as i8, self.level, false)?;
        let best_action = res.best_action.ok_or("no result")?;
        self.play_col(best_action, player, window)?;

        window.map(|w| emit_update(Update::Balance { value: res.score }, w));
        Ok(())
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

        window.map_or(Ok(()), |w| emit_update(Update::State { 
            state: self.state as i8,
            winner: None,
        }, w))?;

        window.map_or(Ok(()), |w| emit_update(Update::Balance { value: 0. }, w))
    }
}

#[cfg(test)]
mod tests {
    use crate::minimax::StateEvaluation;

    use super::*;

    fn evaluate_state(game:&Game, player:CellState) -> Result<StateEvaluation<usize>, String> {
        engine::evaluate_state(
            Option::Some(game.map_values()),
            player as i8,
            game.level,
            false
        )
    }

    #[test]
    fn test_enum() {
        assert_eq!(CellState::P2 as i8, -1);
    }

    #[test]
    fn test_play() {
        let mut g = Game::new(3);
        let (x,o) = (CellState::P1, CellState::P2);
        g.play_col(3, x, None).unwrap();
        g.play_col(5, o, None).unwrap();
        g.play_col(2, x, None).unwrap();
        g.play_col(5, o, None).unwrap();

        let state = evaluate_state(&g, x);
        assert_eq!(state.map(|r| r.best_action).unwrap().unwrap(), 1);

        g.play_col(1, x, None).unwrap();
        g.play_col(0, o, None).unwrap();
        
        let state = evaluate_state(&g, x);
        assert_eq!(state.map(|r| r.best_action).unwrap().unwrap(), 4);
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

        assert_eq!(evaluate_state(&g, x).map(|r| r.best_action).unwrap().unwrap(), 4);
        assert_eq!(evaluate_state(&g, o).map(|r| r.best_action).unwrap().unwrap(), 4);

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

        assert_eq!(evaluate_state(&g, o).map(|r| r.best_action).unwrap().unwrap(), 0);

        g.play_col(6, o, None).unwrap();

        assert_eq!(evaluate_state(&g, x).map(|r| r.best_action).unwrap().unwrap(), 3);        
    }
}