use std::{cell::RefCell, cmp::{max, min}, sync::Arc, vec};
use array2d::Array2D;
use minimax::{Environment, minimize, maximize};

use crate::minimax;

const WIDTH:usize = 8;
const HEIGHT:usize = 7;

const P1:i8 = 1;
const P2:i8 = -1;
const BLANK:i8 = 0;

#[derive(Clone)]
enum Player {
    P1,
    P2,
}

macro_rules! toref {
    ($vector:expr) => (
        ($vector).iter().map(|x| Arc::new(RefCell::new(*x))).collect()
    );
}

macro_rules! gather {
    ($values:expr, $coord_vec:expr) => (
        match $coord_vec.len() > 0 {
            true => Option::Some(($coord_vec).iter().map(|x| Arc::clone(&$values[*x])).collect()),
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

fn check(val:i8, values:&Vec<Arc<RefCell<i8>>>) -> i8 {
    let mut best_score: i8 = 0;
    for i in 4..values.len() {
        let mut score: i8 = 0;

        for v_ref in values[i-4..i].iter() {
            let v:i8 = *(*v_ref).as_ref().borrow();

            if v == -val {
                score = 0;
                break;
            }

            if v == val {
                score += 1;
            }
        }
        best_score = max(score, best_score);
    }
    best_score
}

pub struct Playfield {
    current_player: i8,
    values: Array2D<Arc<RefCell<i8>>>,
    col_heights: [usize; WIDTH],
    last_move: Option<usize>,
    evaluation_score: Option<i8>,
    /**
     * when acessing field sequences[(1,2)], a vector containing sequences of references to cells obtained.
     * for each sequence of the vector, its references are to be iterated and checked for victory condition (four in a row).
     */
    sequences: Array2D<Vec<Vec<Arc<RefCell<i8>>>>>,
    finished: Option<bool>,
}

impl Playfield {
    fn calculate_state(&self, col:usize) -> (bool,i8) {
        let row = self.col_heights[col];
        let val = *self.values[(row, col)].borrow();
        let mut total_score = 0;
        for seq in self.sequences[(row, col)].iter() {
            let score = check(val, seq);
            if score > 3 {
                total_score = 127 * val;
                return (true, total_score);
            }
            total_score += score;
        }
        
        total_score *= val;
        return (
            self.col_heights.iter().map(|x| *x == HEIGHT).fold(true, |a, b| a && b), 
            total_score
        );
    }

    fn update_state(&mut self) -> (bool,i8) {
        let (finished, score) = match self.last_move {
            None => (false, 0),
            Some(col) => self.calculate_state(col)
        };
        self.evaluation_score = Option::Some(score);
        self.finished = Option::Some(finished);
        (finished, score)
    }
}


impl Environment<usize> for Playfield {
    fn evaluate(&mut self) -> i8 {
        self.evaluation_score.unwrap_or_else(|| {
            let (_, score) = self.update_state();
            score
        })
    }
 
    fn apply(&mut self, action:&usize) {        
        let col = *action;
        let h = self.col_heights[col];

        *self.values[(h, col)].borrow_mut() = self.current_player;

        self.col_heights[col] = h + 1;
        self.finished = Option::None;
        self.evaluation_score = Option::None;
    }
 
    fn revert(&mut self, action:&usize) {
        let col = *action;
        let h = self.col_heights[col] - 1;

        *self.values[(h, col)].borrow_mut() = 0;

        self.col_heights[col] = h;
        self.finished = Option::Some(false);
        self.evaluation_score = Option::None;
    }
 
    fn is_finished(&mut self) -> bool {
        self.finished.unwrap_or_else(|| {
            let (finished, _) = self.update_state();
            finished
        })
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

impl Playfield {
    pub fn new() -> Playfield {
        let mut p = Playfield {
            current_player: P1,
            values: Array2D::filled_by_row_major(|| Arc::new(RefCell::new(0)), HEIGHT, WIDTH),
            col_heights: [0; WIDTH],
            last_move: Option::None,
            sequences: Array2D::filled_with(vec![vec![]], HEIGHT, WIDTH),
            finished: Option::Some(false),
            evaluation_score: Option::Some(0),
        };

        for row in 0..HEIGHT {
            for col in 0..WIDTH {
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

    pub fn play_col(&mut self, col:usize) -> Result<(),String> {
        match self.is_finished() {
            true => Result::Err("Game already finished".into()),
            false => {
                self.apply(&col);
                Result::Ok(())
            }
        }
    }

    pub fn get_best_move(&mut self) -> Result<usize,String> {
        match self.current_player {
            P1 => maximize(self, 5).ok_or("Player 1 has no legal move.".into()),
            P2 => minimize(self, 5).ok_or("Player 2 has no legal move.".into()),
            _ => Err("unknown player".into())
        }
    }

    pub fn play_best_move(&mut self) -> Result<usize,String> {
        let col = self.get_best_move()?;
        let _ = self.play_col(col);
        Ok(col)
    }

    fn print_seq(self) {
        self.sequences.enumerate_row_major().for_each(|((r,c), v)| {
            println!("{:?}", (r,c));
            println!("{:?}", v);
            println!("-------------------------");
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macros() {
        // let rows = vec![
        //     vec![ 0, 1, 2, 3, 4, 5, 6, 7],
        //     vec![10,11,12,13,14,15,16,17],
        //     vec![20,21,22,23,24,25,26,27],
        //     vec![30,31,32,33,34,35,36,37],
        //     vec![40,41,42,43,44,45,46,47],
        //     vec![50,51,52,53,54,55,56,57],
        //     vec![60,61,62,63,64,65,66,67],
        // ];

        // let arr = Array2D::from_rows(&rows).unwrap();

        // assert_eq!(horizontal!(arr, 1, 1..=3).iter().map(|x| **x).collect::<Vec<i8>>(), vec![11,12,13]);
        // assert_eq!(vertical!(arr, 5, 2).iter().map(|x| **x).collect::<Vec<i8>>(), vec![2,12,22,32,42,52]);

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
    fn test_unsafe() {
        let mut num = 5;

        let r1 = &num as *const i32;
        let r2 = &mut num as *mut i32;

        unsafe {
            println!("r1 is: {}", *r1);
            println!("r2 is: {}", *r2);
        }
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
    fn test_multi_ref() {
        let values = Array2D::filled_by_row_major(|| Arc::new(RefCell::new(0)), HEIGHT, WIDTH);
        let x1 = Arc::clone(&values[(1,2)]);
        let x2 = Arc::clone(&values[(1,2)]);

        *x1.borrow_mut() += 1;
        assert_eq!(x1, x2);
        assert_eq!(*(*x2).borrow(), 1);
    }

    #[test]
    fn test_macro() {
        assert_eq!(check(1, &toref!(vec![0,0,1,1,1,0,1,1,-1,0,0,0,0])), 3);
        assert_eq!(check(1, &toref!(vec![0,0,-1,1,1,1,1,1,-1,0,0,0,0])), 4);
        assert_eq!(check(1, &toref!(vec![0,0,-1,-1,1,0,0,0,0])), 1);

        assert_eq!(check(-1, &toref!(vec![0,-1,-1,0,0])), 2);
        assert_eq!(check(-1, &toref!(vec![0,-1,-1,1,0])), 0);
        assert_eq!(check(1, &toref!(vec![1,1,0,1,-1,1,0])), 3);

        assert_eq!(check(1, &toref!(vec![-1,-1,1,0])), 0);
        
        assert_eq!(check(1, &toref!(vec![0,0,-1,-1,1,0,0,0,0])), 1);
    }

    #[test]
    fn test_benchmark_ref() {
        use std::time::Instant;
        use rand::Rng;

        let values = Array2D::filled_by_row_major(|| Arc::new(RefCell::new(0)), HEIGHT, WIDTH);
        let x1 = Arc::clone(&values[(1,2)]);
        let x2 = Arc::clone(&values[(4,7)]);
        let mut rng = rand::thread_rng();
        let vals: Vec<f32> = (0..1_000_000).map(|_| rng.gen_range(0.0..1.0)).collect();

        let now = Instant::now();
        for i in vals {
            if i < 0.5 {
                *x1.borrow_mut() += 1;
            } else {
                *x2.borrow_mut() += 1;
            }
        }
        let elapsed = now.elapsed();
        println!("Elapsed: {:.2?}", elapsed);
    }

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