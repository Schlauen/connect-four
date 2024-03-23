use std::{cmp::{max, min}, pin::Pin};

use array2d::Array2D;

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

macro_rules! horizontal {
    ($values:expr, $row:expr, $cols_slice:expr) => {
        {
            ($cols_slice).map(|col| &$values[($row, col as usize)]).collect::<Vec<&i8>>()
        }
    };
}

macro_rules! diagonal_inc {
    ($values:expr, $row:literal, $cols_slice:expr) => {
        {
            ($cols_slice).enumerate().map(|(i, col)| &$values[($row as usize + i, col as usize)]).collect::<Vec<&i8>>()
        }
    };
}

macro_rules! diagonal_dec {
    ($values:expr, $row:literal, $cols_slice:expr) => {
        {
            ($cols_slice).enumerate().map(|(i, col)| &$values[($row as usize - i, col as usize)]).collect::<Vec<&i8>>()
        }
    };
}

macro_rules! vertical {
    ($values:expr, $row:expr, $col:expr) => {
        {
            (0..=$row).map(|r| &$values[(r as usize, $col)]).collect::<Vec<&i8>>()
        }
    };
}

pub struct Playfield<'a> {
    current_player: i8,
    values: Array2D<i8>,
    col_heights: [usize; WIDTH],
    // sequences: Array2D<Vec<Vec<&'a i8>>>
    sequences: Array2D<Vec<Vec<&'a i8>>>,
}

fn check(values:&Array2D<i8>, rc:(usize, usize), val:i8) -> u8 {
    let x = horizontal(values, rc, val);
    // diagonal
    x
}

fn diagonal_1(values:&Array2D<i8>, rc:(usize, usize), val:i8) -> u8 {
    let (r, c) = rc;

    let mut start = r;
    
    let mut can_go = true;
    let mut i = r;
    let mut j = c;

    while can_go {
        start = i;
        if i == 0 || j == 0 {
            break;
        }
        
        if r - start > 2 {
            return 4;
        }

        i -= 1;
        j -= 1;
        can_go &= *&values[(i, j)] == val;
    }
    i = r;
    j = c;

    let mut end = r;
    can_go = true;
    while can_go {
        if i - start > 2 {
            return 4;
        }

        i += 1;
        j += 1;
        end = i;
       
        if i == HEIGHT || j == WIDTH {
            break;
        }
        can_go &= *&values[(i, j)] == val;
    }

    (end - start) as u8
}

fn diagonal_2(values:&Array2D<i8>, rc:(usize, usize), val:i8) -> u8 {
    let (r, c) = rc;

    let mut start = r;
    
    let mut can_go = true;
    let mut i = r;
    let mut j = c;

    while can_go {
        start = i;
        if i == 0 || j == WIDTH {
            break;
        }
        
        if r - start > 2 {
            return 4;
        }

        i -= 1;
        j += 1;
        can_go &= *&values[(i, j)] == val;
    }
    i = r;
    j = c;

    let mut end = r;
    can_go = true;
    while can_go {
        if i - start > 2 {
            return 4;
        }

        i += 1;
        j -= 1;
        end = i;
       
        if i == HEIGHT || j == 0 {
            break;
        }
        can_go &= *&values[(i, j)] == val;
    }

    (end - start) as u8
}

fn horizontal(values:&Array2D<i8>, rc:(usize, usize), val:i8) -> u8 {
    let (r, c) = rc;

    let mut start = c;
    let mut can_go = true;
    let mut i = c;

    // horizontal
    while can_go {
        start = i;
        if i == 0 {
            break;
        }
        
        if c - start > 2 {
            return 4;
        }

        i -= 1;
        can_go &= *&values[(r, i)] == val;
    }
    i = c;

    let mut end = c;
    can_go = true;
    while can_go {
        if i - start > 2 {
            return 4;
        }

        i += 1;
        end = i;
       
        if i == WIDTH {
            break;
        }
        can_go &= *&values[(r, i)] == val;
    }

    (end - start) as u8
}

fn horizontal_2(values:&Vec<&i8>, val:i8) -> u8 {
    let mut score = 0;

    for v in values {
        if score > 3 {
            return 4;
        }
        if **v == val {
            score += 1;
        } else if **v == 0 {
            continue;
        } else {
            return score;
        }
    }

    return score;
}

impl<'a> Playfield<'a> {
    pub fn new() -> Playfield<'a> {
        Playfield {
            current_player: P1,
            values: Array2D::filled_with(0, HEIGHT, WIDTH),
            col_heights: [0; WIDTH],
            sequences: Array2D::filled_with(vec![vec![]], 1, 1),
        }
    }

    fn init(&'a mut self) {
        self.values.enumerate_row_major().for_each(|((r,c), value)| {
            let mut v = Vec::new();

            let start_col = max(0, c-3);
            let end_col = min(WIDTH, c+3);
            
            if end_col - start_col > 2 {
                v.push(horizontal!(self.values, r, start_col..=end_col));
            }

            // diagonal 1
            let mut d1: Vec<&i8> = Vec::new();
            
            for i in 1..4 {
                let rr = r as i32 - i;
                if rr < 0 {
                    break;
                }
                let cc = c as i32 - i;
                if cc < 0 {
                    break;
                }

                d1.push(&self.values[(rr as usize, cc as usize)]);
            }
            d1.push(value);
            for i in 1..4 {
                let rr = r + i;
                if rr >= HEIGHT {
                    break;
                }
                let cc = c + i;
                if cc >= WIDTH {
                    break;
                }
                
                d1.push(&self.values[(rr, cc)]);
            }
            
            if d1.len() > 3 {
                v.push(d1);
            }
            
            // diagonal 2
            let mut d2: Vec<&i8> = Vec::new();
            
            for i in 1..4 {
                let rr = r as i32 - i;
                if rr < 0 {
                    break;
                }
                let cc = c + i as usize ;
                if cc >= WIDTH {
                    break;
                }

                d2.push(&self.values[(rr as usize, cc)]);
            }
            d2.push(value);
            for i in 1..4 {
                let rr = r + i;
                if rr >= HEIGHT {
                    break;
                }
                let cc = c as i32 - i as i32;
                if cc < 0 {
                    break;
                }
                
                d2.push(&self.values[(rr, cc as usize)]);
            }
            
            if d2.len() > 3 {
                v.push(d2);
            }


            let start = max(0, r-3);
            if start < r {
                v.push(vertical!(self.values, r, c))
            }

            if v.is_empty() {
                return;
            }

            self.sequences[(r,c)] = v;
        });
        // let seq = Array2D::from_rows(&vec![
        //     vec![ // row 0
        //         vec![ // col 0
        //             horizontal!(self.values, 0, 0..=3),
        //             diagonal_inc!(self.values, 0, 0..=3),
        //         ],
        //         vec![ // col 1
        //             horizontal!(self.values, 0, 0..=4),
        //             diagonal_inc!(self.values, 0, 1..=4),
        //         ],
        //         vec![ // col 2
        //             horizontal!(self.values, 0, 0..=5),
        //             diagonal_inc!(self.values, 0, 2..=5),
        //         ],
        //         vec![ // col 3
        //             horizontal!(self.values, 0, 0..=6),
        //             diagonal_inc!(self.values, 0, 3..=6),
        //             diagonal_dec!(self.values, 3, 0..=3),
        //         ],
        //         vec![ // col 4
        //             horizontal!(self.values, 0, 1..=7),
        //             diagonal_inc!(self.values, 0, 4..=7),
        //             diagonal_dec!(self.values, 3, 1..=4),
        //         ],
        //         vec![ // col 5
        //             horizontal!(self.values, 0, 2..=7),
        //             diagonal_dec!(self.values, 3, 2..=5),
        //         ],
        //         vec![ // col 6
        //             horizontal!(self.values, 0, 3..=7),
        //             diagonal_dec!(self.values, 3, 3..=6),
        //         ],
        //         vec![ // col 7
        //             horizontal!(self.values, 0, 4..=7),
        //             diagonal_dec!(self.values, 3, 3..=6),
        //         ],
        //     ],
        //     vec![ // row 1
        //         vec![ // col 0
        //             horizontal!(self.values, 1, 0..=3),
        //             diagonal_inc!(self.values, 0, 0..=3),
        //         ],
        //         vec![ // col 1
        //             horizontal!(self.values, 1, 0..=4),
        //             diagonal_inc!(self.values, 0, 0..=4),
        //         ],
        //         vec![ // col 2
        //             horizontal!(self.values, 1, 0..=5),
        //             diagonal_inc!(self.values, 0, 1..=5),
        //             diagonal_dec!(self.values, 3, 0..=3),
        //         ],
        //         vec![ // col 3
        //             horizontal!(self.values, 1, 0..=6),
        //             diagonal_inc!(self.values, 0, 2..=6),
        //             diagonal_dec!(self.values, 4, 0..=4),
        //         ],
        //         vec![ // col 4
        //             horizontal!(self.values, 1, 1..=7),
        //             diagonal_inc!(self.values, 0, 3..=7),
        //             diagonal_dec!(self.values, 4, 1..=5),
        //         ],
        //         vec![ // col 5
        //             horizontal!(self.values, 1, 2..=7),
        //             diagonal_inc!(self.values, 0, 4..=7),
        //             diagonal_dec!(self.values, 4, 2..=6),
        //         ],
        //         vec![ // col 6
        //             horizontal!(self.values, 1, 3..=7),
        //             diagonal_dec!(self.values, 4, 3..=7),
        //         ],
        //         vec![ // col 7
        //             horizontal!(self.values, 1, 4..=7),
        //             diagonal_dec!(self.values, 4, 4..=7),
        //         ],
        //     ],         
        // ]).unwrap();
        // self.sequences = seq;
    } 

    pub fn play_col(&mut self, col:usize) {
        let h = self.col_heights[col];

        self.values[(h, col)] = self.current_player;

        self.col_heights[col] = h + 1;
        
        self.current_player = -self.current_player
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_horizontal() {
        let rows = vec![
            vec![0,1,2,3,4,5,6,7],
            vec![8,9,10,11,12,13,14,15]
        ];
        let arr = Array2D::from_rows(&rows).unwrap();
        assert_eq!(horizontal!(arr, 0, 1..=4), vec![&arr[(0,1)], &arr[(0,2)], &arr[(0,3)], &arr[(0,4)]]);
        assert_eq!(horizontal!(arr, 1, 1..=3), vec![&arr[(1,1)], &arr[(1,2)], &arr[(1,3)]]);
    }

    #[test]
    fn test_diagonal1() {
        let rows = vec![
            vec![ 0, 1, 2, 3, 4, 5, 6, 7],
            vec![10,11,12,13,14,15,16,17],
            vec![20,21,22,23,24,25,26,27],
            vec![30,31,32,33,34,35,36,37],
            vec![40,41,42,43,44,45,46,47],
            vec![50,51,52,53,54,55,56,57],
            vec![60,61,62,63,64,65,66,67],
        ];

        let arr = Array2D::from_rows(&rows).unwrap();

        assert_eq!(horizontal!(arr, 1, 1..=3).iter().map(|x| **x).collect::<Vec<i8>>(), vec![11,12,13]);
        assert_eq!(diagonal_inc!(arr, 2, 0..=4).iter().map(|x| **x).collect::<Vec<i8>>(), vec![20,31,42,53,64]);
        assert_eq!(diagonal_dec!(arr, 4, 3..=6).iter().map(|x| **x).collect::<Vec<i8>>(), vec![43,34,25,16]);
        assert_eq!(vertical!(arr, 5, 2).iter().map(|x| **x).collect::<Vec<i8>>(), vec![2,12,22,32,42,52]);
    }

    #[test]
    fn test_benchmark() {
        let rows = vec![vec![0,1,1,1,0,1,1,1]];
        let arr = Array2D::from_rows(&rows).unwrap();
        
        use std::time::Instant;
        
        let now = Instant::now();
        for _ in 0..100_000 {
            check(&arr, (0,1), 1);
        }
        
        let elapsed = now.elapsed();
        println!("Elapsed: {:.2?}", elapsed); // 9.5 ms
    }

    #[test]
    fn test_benchmark2() {
        let rows = vec![vec![0,1,1,1,0,1,1,1]];
        let arr = Array2D::from_rows(&rows).unwrap();
        
        use std::time::Instant;
        
        
        let s = Array2D::from_rows(&vec![
            vec![ // row 0
                horizontal!(arr, 0, 0..=3), // col 0
                horizontal!(arr, 0, 0..=4), // col 1
            ],
        ]).unwrap();
        
        let now = Instant::now();
        for _ in 0..100_000 {
            horizontal_2(&s[(0,1)], 1);
        }
        
        let elapsed = now.elapsed();
        println!("Elapsed: {:.2?}", elapsed); // 9.5 ms
    }

    #[test]
    fn test_check() {
        let mut rows = vec![vec![0,1,1,1,0,1,0]];
        let mut arr = Array2D::from_rows(&rows).unwrap();
        assert_eq!(3, check(&arr, (0,1), 1));
        assert_eq!(3, check(&arr, (0,2), 1));
        assert_eq!(3, check(&arr, (0,3), 1));
        assert_eq!(1, check(&arr, (0,5), 1));

        rows = vec![vec![0,1,1,1,-1,1,0]];
        arr = Array2D::from_rows(&rows).unwrap();
        assert_eq!(3, check(&arr, (0,1), 1));
        assert_eq!(3, check(&arr, (0,2), 1));
        assert_eq!(3, check(&arr, (0,3), 1));
        assert_eq!(1, check(&arr, (0,5), 1));

        rows = vec![vec![0,-1,-1,-1,1,-1,0]];
        arr = Array2D::from_rows(&rows).unwrap();
        assert_eq!(3, check(&arr, (0,1), -1));
        assert_eq!(3, check(&arr, (0,2), -1));
        assert_eq!(3, check(&arr, (0,3), -1));
        assert_eq!(1, check(&arr, (0,5), -1));

        rows = vec![vec![-1,1,1,1,1,-1,0]];
        arr = Array2D::from_rows(&rows).unwrap();
        assert_eq!(1, check(&arr, (0,0), -1));
        assert_eq!(4, check(&arr, (0,1), 1));
        assert_eq!(4, check(&arr, (0,2), 1));
        assert_eq!(4, check(&arr, (0,3), 1));
        assert_eq!(4, check(&arr, (0,4), 1));
        assert_eq!(1, check(&arr, (0,5), -1));
    }
}