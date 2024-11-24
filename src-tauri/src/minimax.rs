use rand::{seq::*, Rng};
use tauri::utils::config;
use std::{cmp::max, iter::Iterator, time::{self, Instant}};
use ordered_float::NotNan;

/// Implemented methods should in general not call each other.
/// State should be persisted and invalidated if necessary
pub trait Environment {
    /// Evaluate the current environment state by a score ranging from -127. to +127.
    /// Note that this function is called for each state which is to be evaluated. 
    /// It is advisable to highly optimize it for fast execution times.
    fn evaluate(&mut self) -> f32;
    
    /// Returns all valid moves an agent can take in the current environment state.
    /// If an empty list is returned, `self.finished()` must yield true.
    /// An action is identified by a usize.  
    fn actions(&self) -> Vec<usize>;

    /// Changes the environment state
    fn apply(&mut self, action:&usize);

    /// Reverts the action taken. May panic if action was not taken
    fn revert(&mut self, action:&usize);

    /// Determines if the Environment is in a final state. If that is the case, no more actions can be performed.
    fn is_finished(&mut self) -> bool;

    /// Toggles the current player between minimizer and maximizer
    fn swap_players(&mut self);    
}

pub struct StateEvaluation {
    pub best_action:Option<usize>,
    pub ops_count:u128,
    pub score:f32
}

pub struct Config {
    time_limit_millis:Option<u128>,
    max_depth:Option<u8>,
    randomized:bool,
    min_score:f32,
    max_score:f32,
    epsilon:f32,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            time_limit_millis:None,
            max_depth:Some(5),
            randomized:false,
            min_score:-127.,
            max_score:127.,
            epsilon:0.95,
        }
    }
}

impl Config {
    pub fn new(
        time_limit_millis:Option<u128>,
        max_depth:Option<u8>,
        randomized:bool,
        min_score:f32,
        epsilon:f32,
    ) -> Config {
        assert!(
            time_limit_millis.is_some() != max_depth.is_some(),
           "exactly one of time_limit_millis or max_depth has to be specified"
        );
        
        Config {
            time_limit_millis,
            max_depth,
            randomized,
            min_score,
            max_score:-min_score,
            epsilon,
        }
    }

    fn keep_going(&self, now:Instant, level:u8) -> bool {
        match self.time_limit_millis {
            Some(tlm) => now.elapsed().as_millis() < tlm,
            None => level < self.max_depth.unwrap()
        }
    }
}

pub fn minimize(env:&mut impl Environment, config:&Config) -> Option<StateEvaluation> {
    return eval(env, config, -1.0);
} 

pub fn maximize(env:&mut impl Environment, config:&Config) -> Option<StateEvaluation> {
    return eval(env, config, 1.0);
}

#[derive(Clone, Copy)]
struct ActionEvaluation {
    action:usize,
    score:f32,
    exploited:bool,
}

struct Action {
    action:usize,
    score:f32,
}

fn eval(env:&mut impl Environment, config:&Config, player:f32) -> Option<StateEvaluation> {
    if env.is_finished() {
        return None;
    }
    let mut level:u8 = 0;

    let mut actions:Vec<ActionEvaluation> = env.actions().iter().map(|action| ActionEvaluation{
        action:*action, 
        score:config.min_score, 
        exploited:false
    }).collect();

    let now = Instant::now();
    let mut unexploited = true;
    let mut ops_count: u128 = 0;
    while unexploited && config.keep_going(now, level) {
        let mut all_exploited = true;
        let mut max_value = config.min_score;
        let mut alpha = config.min_score.clone();
        let mut beta = config.max_score.clone();

        print!("search until level {:?}. ", level);
        
        actions.iter_mut()
        .for_each(|action_eval| {
            if !action_eval.exploited {
                env.apply(&action_eval.action);
                let (score, exploited, cnt) = deepen(
                    env, 
                    alpha, 
                    beta, 
                    level, 
                    player, 
                    config
                );
                print!("ops {:?}. ", cnt);
                ops_count += cnt;
                action_eval.score = player * score;
                action_eval.exploited = exploited;
                all_exploited &= exploited;
                
                if action_eval.score > max_value {
                    max_value = action_eval.score;
                }
                env.revert(&action_eval.action);
            }
        });
        println!("");
        actions.sort_by_key(|v| NotNan::new(-v.score).unwrap());
        level += 1;
        
        unexploited = !all_exploited;
    }

    // println!("scores: {:?}", actions.clone().into_iter().map(|a| a.score).collect::<Vec<f32>>());
    let best_move: Option<ActionEvaluation> = match config.randomized {
        true => {
            let mut rng = rand::thread_rng();
            actions.into_iter().max_by_key(|i| {
                NotNan::new(i.score * rng.gen_range(0.8..1.2)).unwrap()
            })
        },
        false => actions.into_iter().max_by_key(|i| NotNan::new(i.score).unwrap())
    };

    Option::Some(StateEvaluation {
        best_action:best_move.map(|i| i.action),
        ops_count:ops_count,
        score:player*best_move.map_or(config.min_score, |i| i.score)
    })
}

fn deepen(
    env:&mut impl Environment, 
    alpha:f32,
    beta:f32,
    level:u8,
    player:f32,
    config:&Config
) -> (f32, bool, u128) {
    if level == 0 {
        return (env.evaluate(), env.is_finished(), 1);
    }

    if env.is_finished() {
        return (env.evaluate(), true, 1);
    }

    env.swap_players();

    let mut all_exploited = true;
    let mut ops_count = 0;
    let mut alpha_ = alpha;
    let mut beta_ = beta;
    let actions = env.actions();
    let best_eval = match player.is_sign_positive() {
        true => {
            let mut best_eval = config.min_score;
            for action in actions {
                env.apply(&action);
                let (eval, exploited, cnt) = deepen(env, alpha_.clone(), beta_.clone(), level - 1, -player, config);
                all_exploited &= exploited;
                ops_count += cnt;

                env.revert(&action);

                if eval > best_eval {
                    best_eval = eval;
                }

                if eval > alpha_ {
                    alpha_ = eval;
                }

                if beta_ <= alpha_ {
                    println!("player 1 breaks at {:?}", eval);
                    break;
                }
            }
            best_eval
        },
        false => {
            let mut best_eval = config.max_score;
            for action in actions {
                env.apply(&action);
                let (eval, exploited, cnt) = deepen(env, alpha_, beta_, level - 1, -player, config);
                all_exploited &= exploited;
                ops_count += cnt;

                env.revert(&action);

                if eval < best_eval {
                    best_eval = eval;
                }

                if eval < beta_ {
                    beta_ = eval;
                }

                if beta_ <= alpha_ {
                    println!("player -1 breaks at {:?}", eval);
                    break;
                }
            }
            best_eval
        }
    };

    env.swap_players();
    (config.epsilon*best_eval, all_exploited, ops_count)
}

#[cfg(test)]
mod tests {
    use std::default;

    use float_cmp::assert_approx_eq;
    use indextree::{Arena, NodeId};
    use rand::prelude::*;
    use super::*;

    struct Game {
        arena:Arena<f32>,
        state:NodeId,
    }
    
    impl Environment for Game {
        fn evaluate(&mut self) -> f32 {
            *self.arena.get(self.state).unwrap().get()
        }
     
        fn apply(&mut self, action:&usize) {
            self.state = self.state.children(&self.arena).skip(*action).next().unwrap();
        }
     
        fn revert(&mut self, _action:&usize) {
            self.state = self.state.ancestors(&self.arena).skip(1).next().unwrap();
        }
     
        fn is_finished(&mut self) -> bool {
            self.state.children(&self.arena).next().is_none()
        }
        
        fn actions(&self) -> Vec<usize> {
            self.state.children(&self.arena).enumerate().map(|(i, _)| i).collect()
        }
        
        fn swap_players(&mut self) { }
    }

    #[test]
    fn simple_case() {      
        let mut arena = Arena::new();

        let root = arena.new_node(0.0);
        root.append_value(10.0, &mut arena);
        root.append_value(-5.0, &mut arena);

        let mut game = Game {
            arena:arena,
            state:root,
        };
        let config = Config {..Default::default() };

        let result = maximize(&mut game, &config).unwrap();
        assert_approx_eq!(f32, 10., result.score, ulps=2);
        assert_eq!(2, result.ops_count);
        assert_approx_eq!(f32, -5., minimize(&mut game, &config).unwrap().score, ulps=2);
    }

    #[test]
    fn case_2() {

        //           a
        //     +-----+-----+
        //     |           |
        //     aa          ab
        // +---+---+   +---+---+
        // |   |   |   |   |   |
        // 1  -5   3  -6   ?   ?

        let mut arena = Arena::new();
        
        let aa = arena.new_node(0.0);
        aa.append_value(10., &mut arena);
        aa.append_value(-5., &mut arena);
        aa.append_value(3., &mut arena);

        let ab = arena.new_node(0.);
        ab.append_value(-6., &mut arena);
        ab.append_value(random(), &mut arena);
        ab.append_value(random(), &mut arena);

        let a = arena.new_node(0.);
        a.append(aa, &mut arena);
        a.append(ab, &mut arena);

        let mut game = Game {
            arena:arena,
            state:a,
        };

        let config = Config {epsilon:1., ..Default::default() };
        
        let (score, all_exploited, ops_count) = deepen(&mut game, config.min_score.clone(), 
        config.max_score.clone(), 2, 1., &config);
        assert_approx_eq!(f32, -5., score);
        assert_eq!(4, ops_count);
        assert!(all_exploited);

        // let result = maximize(&mut game, &config).unwrap();
        // assert_approx_eq!(f32, -5., result.score);
        // assert_eq!(7, result.ops_count);

        // assert_approx_eq!(f32, 10., minimize(&mut game, &config).unwrap().score);
    }

    #[test]
    fn case_3() {
        let mut arena = Arena::new();

        //           a                       b
        //     +-----+-----+           +-----+-----+
        //     |           |           |           |
        //     aa          ab          ba          bb
        // +---+---+   +---+---+   +---+---+   +---+---+
        // |   |   |   |   |   |   |   |   |   |   |   |
        // 1  -5   3  -6   15  ?   10  12  3   13  ?   ? 
        
        let aa = arena.new_node(0.);
        aa.append_value(10., &mut arena);
        aa.append_value(-5., &mut arena);
        aa.append_value(3., &mut arena);

        let ab = arena.new_node(0.);
        ab.append_value(-6., &mut arena);
        ab.append_value(15., &mut arena);
        ab.append_value(random(), &mut arena);

        let a = arena.new_node(0.);
        a.append(aa, &mut arena);
        a.append(ab, &mut arena);

        let ba = arena.new_node(0.);
        ba.append_value(10., &mut arena);
        ba.append_value(12., &mut arena);
        ba.append_value(3., &mut arena);

        let bb = arena.new_node(0.);
        bb.append_value(13., &mut arena);
        bb.append_value(random(), &mut arena);
        bb.append_value(random(), &mut arena);

        let b = arena.new_node(0.);
        b.append(ba, &mut arena);
        b.append(bb, &mut arena);

        let root = arena.new_node(0.);
        root.append(a, &mut arena);
        root.append(b, &mut arena);

        let mut game = Game {
            arena:arena,
            state:root,
        };

        let config = Config {epsilon:1.0, ..Default::default() };
        let res = maximize(&mut game, &config).unwrap();
        assert_approx_eq!(f32, 12.0, res.score);
        assert_eq!(14, res.ops_count);
    }

    #[test]
    fn case_4() {      
        let mut arena = Arena::new();

        let a = arena.new_node(0.);
        a.append_value(-100., &mut arena);


        let c = arena.new_node(0.);
        c.append_value(-100., &mut arena);

        let root = arena.new_node(0.0);
        root.append(a, &mut arena);
        root.append_value(-50.0, &mut arena);
        root.append(c, &mut arena);

        let mut game = Game {
            arena:arena,
            state:root,
        };
        let config = Config {..Default::default() };
        let result = maximize(&mut game, &config).unwrap();
        assert_approx_eq!(f32, -50., result.score, ulps=2);
        assert_eq!(4, result.ops_count);
        assert_eq!(2, result.best_action.unwrap());
    }
}
