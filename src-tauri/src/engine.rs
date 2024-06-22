use std::{cmp::{max, min}, collections::{HashMap, VecDeque}, sync::RwLock, vec};
use array2d::Array2D;
use minimax::{Environment, minimize, maximize};

use crate::minimax;

pub const WIDTH:usize = 8;
pub const HEIGHT:usize = 7;
pub const TOTAL_FIELDS:usize = WIDTH * HEIGHT;

const MAX_SCORE:i8 = 127;
const P1:i8 = 1;
const P2:i8 = -1;
const BLANK:i8 = 0;

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
        (start..=$row).map(|r| (r as usize, $col)).collect::<Vec<(usize, usize)>>()
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

fn check(val:i8, values:&Vec<*mut i8>) -> i8 {
    let mut best_score: i8 = 0;
    for i in 4..values.len() {
        let mut score: i8 = 0;

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

fn is_full(col_heights:[usize; WIDTH]) -> bool {
    col_heights.iter().map(|x| *x == HEIGHT).fold(true, |a, b| a && b)
}

#[derive(Clone)]
pub struct EvaluationResult {
    pub score: i8,
    pub finished: bool,
    pub winner: Option<i8>,
}

struct ConnectFour {
    current_player: i8,
    values: Array2D<i8>,
    col_heights: [usize; WIDTH],
    move_history: VecDeque<usize>,
    evaluation_result: Option<EvaluationResult>,
    set_fields: usize,

    /**
     * when acessing field sequences[(1,2)], a vector containing sequences of references to cells obtained.
     * for each sequence of the vector, its references are to be iterated and checked for victory condition (four in a row).
     */
    sequences: Array2D<Vec<Vec<*mut i8>>>,
}

impl ConnectFour {
    fn calculate_state(&self, col:usize) -> (bool,i8) {
        let row = self.col_heights[col];
        let val = self.values[(row, col)];
        let mut total_score = 0;
        for seq in self.sequences[(row, col)].iter() {
            let score = check(val, seq);
            if score > 3 {
                total_score = MAX_SCORE * val;
                return (true, total_score);
            }
            total_score += score;
        }
        
        total_score *= val;
        (
            self.set_fields >= TOTAL_FIELDS, 
            total_score
        )
    }

    fn create_evaluation(&mut self) {
        if self.evaluation_result.is_none() {
            let (finished, score) = self.move_history.back()
                .map(|col| self.calculate_state(*col))
                .unwrap_or((false, 0));

            self.evaluation_result = Option::Some(EvaluationResult {
                score,
                finished,
                winner: match finished {
                    true => Option::Some(score / MAX_SCORE),
                    false => Option::None
                }
            });
        }
    }
}

impl Environment<usize> for ConnectFour {
    fn evaluate(&mut self) -> i8 {
        self.create_evaluation();
        self.evaluation_result.as_ref().expect("not evaluated").score
    }
 
    fn apply(&mut self, action:&usize) {        
        let col = *action;
        let h = self.col_heights[col];

        self.values[(h, col)] = self.current_player;

        self.col_heights[col] = h + 1;
        self.set_fields += 1;

        self.move_history.push_back(col);
        self.evaluation_result = Option::None;
    }
 
    fn revert(&mut self, action:&usize) {
        let col = *action;
        let h = self.col_heights[col] - 1;

        self.values[(h, col)] = 0;

        self.col_heights[col] = h;
        self.set_fields -= 1;
        
        self.move_history.pop_back();
        self.evaluation_result = Option::None;
    }
 
    fn is_finished(&mut self) -> bool {
        self.create_evaluation();
        self.evaluation_result.as_ref().expect("not evaluated").finished
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
    pub fn new(values: Option<Array2D<i8>>, col_heights:Option<[usize; WIDTH]>, current_player:i8) -> ConnectFour {
        let mut p = ConnectFour {
            current_player: current_player,
            values: values.unwrap_or(Array2D::filled_with(0, HEIGHT, WIDTH)),
            col_heights: col_heights.unwrap_or([0; WIDTH]),
            move_history: VecDeque::with_capacity(TOTAL_FIELDS),
            sequences: Array2D::filled_with(vec![vec![]], HEIGHT, WIDTH),
            evaluation_result: Option::None,
            set_fields: 0,
        };

        for row in 0..HEIGHT {
            for col in 0..WIDTH {
                if p.values[(row,col)] != 0 {
                    p.col_heights[col] += 1;
                    p.set_fields += 1;
                }

                let mut sequences = Vec::new();
                println!("{:?}", (row,col));

                gather!(p.values, v_tup_seq!(row,col)).map(|refs| sequences.push(refs));
                gather!(p.values, h_tup_seq!(row,col)).map(|refs| sequences.push(refs));
                gather!(p.values, rdiag_tup_seq!(row,col)).map(|refs| sequences.push(refs));
                gather!(p.values, ldiag_tup_seq!(row,col)).map(|refs| sequences.push(refs));

                p.sequences[(row,col)] = sequences;
            }
        }
        p
    }

    fn print_seq(self) {
        self.sequences.enumerate_row_major().for_each(|((r,c), v)| {
            println!("{:?}", (r,c));
            println!("{:?}", v);
            println!("-------------------------");
        });
    }
}

pub fn get_best_move(values: Option<Array2D<i8>>, col_heights:Option<[usize; WIDTH]>, current_player:i8) -> Result<usize,String> {
    let mut g = ConnectFour::new(values.map(|x| x.clone()), col_heights, current_player);
    match g.current_player {
        P1 => maximize(&mut g, 5).ok_or("Player 1 has no legal move.".into()),
        P2 => minimize(&mut g, 5).ok_or("Player 2 has no legal move.".into()),
        _ => Err("unknown player".into())
    }
}

/// Returns Option::None when not finished, Option::Some(0) if the game is a draw, 
/// or Option::Some(1) if maximizer has won the game or Option::Some(-1) if minimizer won.
pub fn get_evaluation(values: Array2D<i8>, col_heights:[usize; WIDTH], current_player:i8) -> EvaluationResult {
    let mut g = ConnectFour::new(
        Option::Some(values),
        Option::Some(col_heights), 
        current_player
    );
    g.create_evaluation();
    g.evaluation_result.expect("not evaluated")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macros() {
        assert_eq!(rdiag_tup_seq!(2,1), vec![(1,0),(2,1),(3,2),(4,3),(5,4)]);
        assert_eq!(rdiag_tup_seq!(5,5), vec![(2,2),(3,3),(4,4),(5,5),(6,6)]);
        assert_eq!(rdiag_tup_seq!(4,6), vec![(1,3),(2,4),(3,5),(4,6),(5,7)]);
        assert_eq!(rdiag_tup_seq!(2,3), vec![(0,1),(1,2),(2,3),(3,4),(4,5),(5,6)]);

        assert_eq!(ldiag_tup_seq!(4,1), vec![(1,4),(2,3),(3,2),(4,1),(5,0)]);
        assert_eq!(ldiag_tup_seq!(1,5), vec![(0,6),(1,5),(2,4),(3,3),(4,2)]);

        assert_eq!(h_tup_seq!(1,5), vec![(1,2),(1,3),(1,4),(1,5),(1,6),(1,7)]);

        assert_eq!(v_tup_seq!(2,5), vec![(0,5),(1,5),(2,5)]);
        assert_eq!(v_tup_seq!(0,0), vec![(0,0)]);
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

    /* #[test]
    fn test_2() {
        let mut p = Playfield::new();

        assert_eq!(p.play_col(0), 1);
        assert_eq!(p.play_col(0), -1);
        assert_eq!(p.play_col(1), 2);
        assert_eq!(p.play_col(3), -1);
        assert_eq!(p.play_col(4), 1);
        assert_eq!(p.play_col(0), -2);
        assert_eq!(p.play_col(3), 2);
        assert_eq!(p.play_col(0), -5);
        assert_eq!(p.play_col(0), 1);
        assert_eq!(p.play_col(4), -2);
        assert_eq!(p.play_col(4), 2);
        assert_eq!(p.play_col(5), -2);
    }
    */

    #[test]
    fn test_macro() {
        assert_eq!(check(1, &toref!(vec![0i8,0,1,1,1,0,1,1,-1,0,0,0,0])), 3);
        assert_eq!(check(1, &toref!(vec![0i8,0,-1,1,1,1,1,1,-1,0,0,0,0])), 4);
        assert_eq!(check(1, &toref!(vec![0i8,0,-1,-1,1,0,0,0,0])), 1);

        assert_eq!(check(-1, &toref!(vec![0i8,-1,-1,0,0])), 2);
        assert_eq!(check(-1, &toref!(vec![0i8,-1,-1,1,0])), 0);
        assert_eq!(check(1, &toref!(vec![1i8,1,0,1,-1,1,0])), 3);

        assert_eq!(check(1, &toref!(vec![-1i8,-1,1,0])), 0);
        
        assert_eq!(check(1, &toref!(vec![0i8,0,-1,-1,1,0,0,0,0])), 1);
    }

    // #[test]
    // fn test_benchmark_ref() {
    //     use std::time::Instant;
    //     use rand::Rng;

    //     let values = Array2D::filled_by_row_major(|| Arc::new(RefCell::new(0)), HEIGHT, WIDTH);
    //     let x1 = Arc::clone(&values[(1,2)]);
    //     let x2 = Arc::clone(&values[(4,7)]);
    //     let mut rng = rand::thread_rng();
    //     let vals: Vec<f32> = (0..1_000_000).map(|_| rng.gen_range(0.0..1.0)).collect();

    //     let now = Instant::now();
    //     for i in vals {
    //         if i < 0.5 {
    //             *x1.borrow_mut() += 1;
    //         } else {
    //             *x2.borrow_mut() += 1;
    //         }
    //     }
    //     let elapsed = now.elapsed();
    //     println!("Elapsed: {:.2?}", elapsed);
    // }

    #[test]
    fn test_benchmark_index() {
        use std::time::Instant;
        use rand::Rng;

        let mut values = Array2D::filled_with(0, HEIGHT, WIDTH);
        let mut rng = rand::thread_rng();
        let vals: Vec<f32> = (0..1_000_000).map(|_| rng.gen_range(0.0..1.0)).collect();

        let x1 = (1,2);
        let x2 = (4,7);
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