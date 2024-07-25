use rand::seq::*;
use std::{iter::Iterator, time::Instant};
use ordered_float::NotNan;

pub const MAX_SCORE:f32 = 127.;
pub const MIN_SCORE:f32 = -127.;

const LAMBDA:f32 = 0.95;

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
    pub ops_count:u32,
    pub score:f32
}

pub fn minimize(env:&mut impl Environment, time_limit_millis:u128, randomized:bool) -> Option<StateEvaluation> {
    return eval(env, time_limit_millis, randomized, -1.0);
} 

pub fn maximize(env:&mut impl Environment, time_limit_millis:u128, randomized:bool) -> Option<StateEvaluation> {
    return eval(env, time_limit_millis, randomized, 1.0);
}

#[derive(Clone, Copy)]
struct ActionEvaluation {
    action:usize,
    score:f32,
    exploited:bool,
}

fn eval(env:&mut impl Environment, time_limit_millis:u128, randomized:bool, player:f32) -> Option<StateEvaluation> {
    if time_limit_millis == 0 || env.is_finished() {
        return None;
    }
    let mut level:u8 = 0;

    let mut actions:Vec<ActionEvaluation> = env.actions().iter().map(|action| ActionEvaluation{
        action:*action, 
        score:MIN_SCORE, 
        exploited:false
    }).collect();

    let now = Instant::now();
    let mut unexploited = true;
    while unexploited && now.elapsed().as_millis() < time_limit_millis {
        let mut all_exploited = true;
        let mut max_value = MIN_SCORE;
        actions.iter_mut()
        .for_each(|action_eval| {
            if !action_eval.exploited {
                env.apply(&action_eval.action);
                let (score, exploited) = deepen(env, -max_value, level, -player); 
                
                action_eval.score = score;
                action_eval.exploited = exploited;
                all_exploited &= exploited;
                
                if action_eval.score > max_value {
                    max_value = action_eval.score;
                }
                env.revert(&action_eval.action);
            }
        });
        actions.sort_by_key(|v| NotNan::new(-v.score).unwrap());
        level += 1;

        unexploited = !all_exploited;
    }

    let best_move: Option<ActionEvaluation> = match randomized {
        true => actions.choose_weighted(&mut rand::thread_rng(),|i| i.score).ok().map(|i| *i),
        false => actions.into_iter().max_by_key(|i| NotNan::new(i.score).unwrap())
    };

    Option::Some(StateEvaluation {
        best_action:best_move.map(|i| i.action),
        ops_count:1,
        score:player*best_move.map_or(MIN_SCORE, |i| i.score)
    })
}

fn deepen(env:&mut impl Environment, b:f32, level:u8, player:f32) -> (f32, bool) {
    if level == 0 {
        return (-player*env.evaluate(), env.is_finished());
    }

    if env.is_finished() {
        return (-player*env.evaluate(), true);
    }

    env.swap_players();
    
    // current player can achieve at least this score (or higher)
    let mut max_score = MIN_SCORE;

    let actions = env.actions();
    let mut all_exploited = true;
    
    for action in actions {
        env.apply(&action);
        let (score, exploited) = deepen(env, -max_score, level - 1, -player);

        all_exploited &= exploited;

        env.revert(&action);

        if score > max_score {
            max_score = score;
            if max_score >= b {
                // found a better move than the opponent's best move.
                // Hence, the opponent won't let the current situation happen and branch can be pruned for the current level
                env.swap_players();
                return (MIN_SCORE, all_exploited);
            }
        }
    }

    env.swap_players();
    (-LAMBDA*max_score, all_exploited)
}

#[cfg(test)]
mod tests {
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

        // maximizer
        assert_approx_eq!(f32, 9.5, -deepen(&mut game, MAX_SCORE, 10, 1.0).0, ulps=2);

        // minimizer
        assert_approx_eq!(f32, -4.75, deepen(&mut game, MAX_SCORE, 10, -1.0).0, ulps=2);

        assert_approx_eq!(f32, 10., maximize(&mut game, 10, false).unwrap().score, ulps=2);
        assert_approx_eq!(f32, -5., minimize(&mut game, 10, false).unwrap().score, ulps=2);
    }

    #[test]
    fn case_2() {
        let mut arena = Arena::new();
        
        let aa = arena.new_node(0.0);
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

        let mut game = Game {
            arena:arena,
            state:a,
        };

        // maximizer
        assert_approx_eq!(f32, -4.5125, -deepen(&mut game, MAX_SCORE, 10, 1.0).0, ulps=2);

        // minimizer
        assert_approx_eq!(f32, 9.025, deepen(&mut game, MAX_SCORE, 10, -1.0).0, ulps=2);

        assert_approx_eq!(f32, -4.75, maximize(&mut game, 10, false).unwrap().score);
        assert_approx_eq!(f32, 9.5, minimize(&mut game, 10, false).unwrap().score);
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

        assert_approx_eq!(f32, 10.2885, -deepen(&mut game, MAX_SCORE, 10, 1.0).0, ulps=2);
        let res = maximize(&mut game, 1000, false).unwrap();
        println!("{:?}", res.ops_count);
        assert_approx_eq!(f32, 10.83, res.score);
    }
}
