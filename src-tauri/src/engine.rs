use std::{cmp::{max, min}, collections::{HashMap, VecDeque}, sync::RwLock, vec};
use array2d::Array2D;
use minimax::{Environment, minimize, maximize, MIN_SCORE, MAX_SCORE};

use crate::minimax;

pub const WIDTH:usize = 7;
pub const HEIGHT:usize = 6;
pub const TOTAL_FIELDS:usize = WIDTH * HEIGHT;

const P1:i8 = 1;
const P2:i8 = -1;

// static mut STATES:RwLock<HashMap<i32, i8>> = RwLock::new(HashMap::new());

macro_rules! toref {
    ($vector:expr) => (
        (0..$vector.len()).map(|i| &mut $vector[i] as *mut i8).collect()
    );
}

macro_rules! gather {
    ($values:expr, $coord_vec:expr) => (
        match $coord_vec.len() > 0 {
            true => Option::Some(($coord_vec).iter().map(|x| &mut $values[*x] as *mut i8).collect()),
            false => Option::None
        }
    );
}

macro_rules! h_tup_seq {
    ($row:expr, $col:expr) => ({
        let start:usize = max(0, ($col as i8)-3) as usize;
        let end = min(WIDTH, $col+4);
        (start..end).map(|c| ($row, c as usize)).collect::<Vec<(usize, usize)>>()
    });
}

macro_rules! v_tup_seq {
    ($row:expr, $col:expr) => ({
        let start:usize = max(0, ($row as i8)-3) as usize;
        let end = min(HEIGHT, $row+4);
        (start..end).map(|r| (r as usize, $col)).collect::<Vec<(usize, usize)>>()
    });
}

macro_rules! rdiag_tup_seq {
    ($row:expr, $col:expr) => ({
        let d = min(min($row, $col), 3);
        let mut r = $row - d;
        let mut c = $col - d;

        let mut values: Vec<(usize, usize)> = Vec::new();
        for _ in 0..d+4 {
            if r >= HEIGHT || c >= WIDTH {
                break;
            }
            values.push((r, c));
            r += 1;
            c += 1;
        }

        if values.len() < 4 {
            values.clear();
        }
        values
    });
}

macro_rules! ldiag_tup_seq {
    ($row:expr, $col:expr) => ({
        let col_ = WIDTH-1-$col;
        let values:Vec<(usize, usize)> = rdiag_tup_seq!($row, col_).iter().map(|(r,c)| (*r, WIDTH-1-c)).collect();
        values
    });
}

fn check(val:i8, values:&Vec<*mut i8>) -> u8 {
    let mut best_score: u8 = 0;
    for i in 4..=values.len() {
        let mut score: u8 = 0;

        for v_ref in values[i-4..i].iter() {
            unsafe {
                let v = *(*v_ref);
                if v == -val {
                    score = 0;
                    break;
                }

                if v == val {
                    score += 1;
                }
            }
        }
        best_score = max(score, best_score);
    }
    best_score
}

#[derive(Clone)]
pub struct Eval {
    pub score: f32,
    pub finished: bool,
    pub winner: Option<i8>,
}

pub struct EvaluationResult {
    pub eval: Eval,
    pub winning_cells: Option<Vec<(usize, usize)>>,
}

struct ConnectFour {
    current_player: i8,
    values: Array2D<i8>,
    col_heights: [usize; WIDTH],
    evaluation_result: Option<Eval>,
    set_fields: usize,
    last_action: Option<usize>,

    /**
     * when acessing field sequences[(1,2)], a vector containing sequences of references to cells obtained.
     * for each sequence of the vector, its references are to be iterated and checked for victory condition (four in a row).
     */
    sequences: Array2D<Vec<Vec<*mut i8>>>,
}

impl ConnectFour {
    fn calculate_state(&self, col:usize) -> Eval {
        let row = self.col_heights[col] - 1;
        let val = self.values[(row, col)];
        let mut total_score = 0.;
        let mut len: u8 = 0;
        for seq in self.sequences[(row, col)].iter() {
            let score = check(val, seq);
            if score > 0 {
                len += 1;
            }
            if score > 3 {
                return Eval {
                    score: MAX_SCORE * val as f32,
                    finished: true,
                    winner: Some(val)
                };
            }
            total_score += score as f32;
        }
        
        // make sure the played field itself counts as only 1
        if len > 1 {
            total_score -= (len - 1) as f32;
        }
        total_score *= val as f32;
        Eval {
            score: total_score,
            finished: self.set_fields >= TOTAL_FIELDS,
            winner: None
        }
    }

    fn eval(&mut self) -> Eval {
        match &self.evaluation_result {
            Some(res) => res.clone(),
            None => {
                self.last_action.map_or(
                    Eval {
                        score: 0.,
                        winner: None,
                        finished: false,
                    },
                    |a| self.calculate_state(a)
                )
            }
        }
    }
}

impl Environment<usize> for ConnectFour {
    fn evaluate(&mut self) -> f32 {
        self.eval().score
    }
 
    fn apply(&mut self, action:&usize) {        
        let col = *action;
        let h = self.col_heights[col];

        self.values[(h, col)] = self.current_player;

        self.col_heights[col] = h + 1;
        self.set_fields += 1;

        self.last_action = Option::Some(col);
        self.evaluation_result = Option::None;
    }
 
    fn revert(&mut self, action:&usize) {
        let col = *action;
        let h = self.col_heights[col] - 1;

        self.values[(h, col)] = 0;

        self.col_heights[col] = h;
        self.set_fields -= 1;
        
        self.last_action = Option::None;
        self.evaluation_result = Option::None;
    }
 
    fn is_finished(&mut self) -> bool {
        self.eval().finished
    }
    
    fn actions(&self) -> Vec<usize> {
        self.col_heights.iter().enumerate().filter_map(|(i, h)| match *h < HEIGHT {
            false => Option::None,
            true => Option::Some(i)
        }).collect()
    }
    
    fn swap_players(&mut self) {
        self.current_player *= -1;
    }
}

impl ConnectFour {
    pub fn new(values: Option<Array2D<i8>>, current_player:i8) -> ConnectFour {
        let mut p = ConnectFour {
            current_player: current_player,
            values: values.unwrap_or(Array2D::filled_with(0, HEIGHT, WIDTH)),
            col_heights: [0; WIDTH],
            sequences: Array2D::filled_with(vec![vec![]], HEIGHT, WIDTH),
            evaluation_result: Option::None,
            set_fields: 0,
            last_action: Option::None
        };

        for row in 0..HEIGHT {
            for col in 0..WIDTH {
                if p.values[(row,col)] != 0 {
                    p.col_heights[col] += 1;
                    p.set_fields += 1;
                }

                let mut sequences = Vec::new();
                gather!(p.values, v_tup_seq!(row,col)).map(|refs| sequences.push(refs));
                gather!(p.values, h_tup_seq!(row,col)).map(|refs| sequences.push(refs));
                gather!(p.values, rdiag_tup_seq!(row,col)).map(|refs| sequences.push(refs));
                gather!(p.values, ldiag_tup_seq!(row,col)).map(|refs| sequences.push(refs));

                p.sequences[(row,col)] = sequences;
            }
        }
        p
    }
}

pub fn get_best_move(values: Option<Array2D<i8>>, current_player:i8, level:u8) -> Result<usize,String> {
    let mut g = ConnectFour::new(values.map(|x| x.clone()), current_player);
    match g.current_player {
        P1 => maximize(&mut g, level).ok_or("Player 1 has no legal move.".into()),
        P2 => minimize(&mut g, level).ok_or("Player 2 has no legal move.".into()),
        _ => Err("unknown player".into())
    }
}

pub fn evaluate_action(state:(Array2D<i8>, i8), action:usize) -> EvaluationResult {
    let (values, current_player) = state;
    let mut g = ConnectFour::new(
        Option::Some(values),
        current_player
    );
    g.last_action = Option::Some(action);
    let result = g.eval();

    let winning_cells = result.winner.map(|val| {
        let check_ = |tup_seq:Vec<(usize,usize)>| {
            let mut seq:Vec<(usize,usize)> = Vec::new();
            for rc in tup_seq {
                if g.values[rc] == val {
                    seq.push(rc);
                } else {
                    seq.clear();
                }
    
                if seq.len() == 4 {
                    return Option::Some(seq);
                }
            }
            Option::None
        };
        let row = g.col_heights[action] - 1;
        check_(rdiag_tup_seq!(row, action))
        .or_else(|| check_(ldiag_tup_seq!(row, action)))
        .or_else(|| check_(h_tup_seq!(row, action)))
        .or_else(|| check_(v_tup_seq!(row, action))).expect("no sequence of four found")
    });
    EvaluationResult {
        eval: result,
        winning_cells
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macros() {
        assert_eq!(rdiag_tup_seq!(2,1), vec![(1,0),(2,1),(3,2),(4,3),(5,4)]);
        assert_eq!(rdiag_tup_seq!(4,4), vec![(1,1),(2,2),(3,3),(4,4),(5,5)]);
        assert_eq!(rdiag_tup_seq!(4,6), vec![(1,3),(2,4),(3,5),(4,6)]);
        assert_eq!(rdiag_tup_seq!(2,3), vec![(0,1),(1,2),(2,3),(3,4),(4,5),(5,6)]);

        assert_eq!(ldiag_tup_seq!(4,1), vec![(1,4),(2,3),(3,2),(4,1),(5,0)]);
        assert_eq!(ldiag_tup_seq!(1,5), vec![(0,6),(1,5),(2,4),(3,3),(4,2)]);

        assert_eq!(h_tup_seq!(1,5), vec![(1,2),(1,3),(1,4),(1,5),(1,6)]);

        assert_eq!(v_tup_seq!(2,5), vec![(0,5),(1,5),(2,5),(3,5),(4,5),(5,5)]);
        assert_eq!(v_tup_seq!(0,0), vec![(0,0),(1,0),(2,0),(3,0)]);
    }

    #[test]
    fn test_benchmark_unsafe() {
        use std::time::Instant;
        use rand::Rng;

        let mut values: Array2D<i32> = Array2D::filled_with(0, HEIGHT, WIDTH);
        values[(0,0)] += 1;
        assert_eq!(1, values[(0,0)]);
        let x1 = &mut values[(1,2)] as *mut i32;
        let x2 = &mut values[(1,2)] as *mut i32;

        unsafe {
            *x1 += 1;
            assert_eq!(1, *x2);
        }

        let mut rng: rand::prelude::ThreadRng = rand::thread_rng();
        let vals: Vec<f32> = (0..1_000_000).map(|_| rng.gen_range(0.0..1.0)).collect();

        let now = Instant::now();
        for i in vals {
            if i < 0.5 {
                unsafe {
                    *x1 += 1;
                }
                
            } else {
                unsafe {
                    *x2 += 1;
                }
            }
        }
        let elapsed = now.elapsed();
        println!("Elapsed: {:.2?}", elapsed);
    }

    #[test]
    fn test_2() {
        let mut p = ConnectFour::new(Option::None, P1);
        let mut play_col = |col|  {
            p.apply(col);
            p.swap_players();
            p.evaluate()
        };

        assert_eq!(play_col(&0), 1.);
        assert_eq!(play_col(&0), -1.);
        assert_eq!(play_col(&1), 2.);
        assert_eq!(play_col(&3), -1.);
        assert_eq!(play_col(&4), 1.);
        assert_eq!(play_col(&0), -2.);
        assert_eq!(play_col(&3), 2.);
        assert_eq!(play_col(&0), -4.);
        assert_eq!(play_col(&0), 2.);
        assert_eq!(play_col(&4), -2.);
        assert_eq!(play_col(&4), 2.);
        assert_eq!(play_col(&5), -2.);
    }

    #[test]
    fn test_col_height() {
        let mut p = ConnectFour::new(Option::None, P1);
        let mut play_col = |col|  {
            p.apply(col);
            p.swap_players();
            p.evaluate();
            p.col_heights[*col]
        };

        assert_eq!(play_col(&0), 1);
        assert_eq!(play_col(&0), 2);
        assert_eq!(play_col(&1), 1);
        assert_eq!(play_col(&3), 1);
        assert_eq!(play_col(&4), 1);
        assert_eq!(play_col(&0), 3);
        assert_eq!(play_col(&3), 2);
        assert_eq!(play_col(&0), 4);
        assert_eq!(play_col(&0), 5);
        assert_eq!(play_col(&4), 2);
        assert_eq!(play_col(&4), 3);
        assert_eq!(play_col(&5), 1);

        let mut revert_col = |col|  {
            p.revert(col);
            p.swap_players();
            p.evaluate();
            p.col_heights[*col]
        };
        assert_eq!(revert_col(&0), 4);
        assert_eq!(revert_col(&0), 3);
        assert_eq!(revert_col(&1), 0);
        assert_eq!(revert_col(&3), 1);
    }

    #[test]
    fn test_macro() {
        assert_eq!(check(1, &toref!(vec![0i8,0,1,1,1,0,1,1,-1,0,0,0,0])), 3);
        assert_eq!(check(1, &toref!(vec![0i8,0,-1,1,1,1,1,1,-1,0,0,0,0])), 4);
        assert_eq!(check(1, &toref!(vec![0i8,0,-1,-1,1,0,0,0,0])), 1);
        assert_eq!(check(-1, &toref!(vec![0i8,-1,-1,0,0])), 2);
        assert_eq!(check(-1, &toref!(vec![0i8,-1,-1,1,0])), 0);
        assert_eq!(check(1, &toref!(vec![1i8,1,0,1,-1,1,0])), 3);
        assert_eq!(check(1, &toref!(vec![1i8,0,0,0])), 1);
        assert_eq!(check(1, &toref!(vec![-1i8,-1,1,0])), 0);
        assert_eq!(check(1, &toref!(vec![0i8,0,-1,-1,1,0,0,0,0])), 1);
    }

    #[test]
    fn test_benchmark_index() {
        use std::time::Instant;
        use rand::Rng;

        let mut values = Array2D::filled_with(0, HEIGHT, WIDTH);
        let mut rng = rand::thread_rng();
        let vals: Vec<f32> = (0..1_000_000).map(|_| rng.gen_range(0.0..1.0)).collect();

        let x1 = (1,2);
        let x2 = (4,6);
        let now = Instant::now();
        for i in vals {
            if i < 0.5 {
                values[x1] += 1;
            } else {
                values[x2] += 1;
            }
        }
        let elapsed = now.elapsed();
        println!("Elapsed: {:.2?}", elapsed);
    }
}