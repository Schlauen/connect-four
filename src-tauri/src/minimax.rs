use rand::seq::*;
use std::iter::Iterator;
use ordered_float::NotNan;

pub const MAX_SCORE:f32 = 127.;
pub const MIN_SCORE:f32 = -127.;
const LAMBDA:f32 = 0.95;

/// Implemented methods should in general not call each other.
/// State should be persisted and invalidated if necessary
pub trait Environment<T:Copy> {
    /// Evaluate the current environment state
    fn evaluate(&mut self) -> f32;
    
    /// Returns all valid moves an agent can take in the current environment state.
    /// If an empty list is returned, `self.finished()` must yield true. 
    fn actions(&self) -> Vec<T>;

    /// Changes the environment state
    fn apply(&mut self, action:&T);

    /// Reverts the action taken. May panic if action was not taken
    fn revert(&mut self, action:&T);

    /// Determines if the Environment is in a final state. If that is the case, no more actions can be performed.
    fn is_finished(&mut self) -> bool;

    /// Toggles the current player between minimizer and maximizer
    fn swap_players(&mut self);
}

pub struct StateEvaluation<T> {
    pub best_action:Option<T>,
    pub ops_count:u32,
    pub score:f32
}

pub fn minimize<T:Copy>(env:&mut impl Environment<T>, level:u8, randomized:bool) -> Option<StateEvaluation<T>> {
    return f(env, level, randomized, -1.0);
} 

pub fn maximize<T:Copy>(env:&mut impl Environment<T>, level:u8, randomized:bool) -> Option<StateEvaluation<T>> {
    return f(env, level, randomized, 1.0);
} 

fn f<T:Copy>(env:&mut impl Environment<T>, level:u8, randomized:bool, player:f32) -> Option<StateEvaluation<T>> {
    if level == 0 || env.is_finished() {
        return None;
    }

    let mut ops:u32 = 0;
    let mut max_value = MIN_SCORE;
    
    let iter = env.actions().into_iter().map(|action| {
        ops += 1;
        env.apply(&action);
        let value = -ff(env, -max_value, level - 1, &mut ops, -player);
        if value > max_value {
            max_value = value;
        }
        env.revert(&action);
        (action, value)
    });

    let best_move: Option<(T, f32)> = match randomized {
        true => iter.collect::<Vec<(T,f32)>>().choose_weighted(&mut rand::thread_rng(), |i| i.1).ok().map(|i| *i),
        false => iter.max_by_key(|i| NotNan::new(i.1).unwrap())
    };

    Option::Some(StateEvaluation {
        best_action:best_move.map(|i| i.0),
        ops_count:ops,
        score:player*best_move.map_or(MIN_SCORE, |i| i.1)
    })
}

fn ff<T:Copy>(env:&mut impl Environment<T>, b:f32, level:u8, ops:&mut u32, player:f32) -> f32 {
    if level == 0 || env.is_finished() {
         return player*env.evaluate();
    }

    env.swap_players();
    
    // current player can achieve at least this score (or higher)
    let mut max_value = MIN_SCORE;
    
    for action in env.actions() {
        *ops += 1;
        env.apply(&action);
        let value = -LAMBDA*ff(env, -max_value, level - 1, ops, -player);
        env.revert(&action);

        if value > max_value {
            max_value = value;
            if max_value >= b {
                // found a better move than the opponent's best move.
                // Hence, the opponent won't let the current situation happen and branch can be pruned
                env.swap_players();
                return MAX_SCORE;
            }
        }
    }

    env.swap_players();
    max_value
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
    
    impl Environment<usize> for Game {
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
        assert_approx_eq!(f32, 9.5, ff(&mut game, MAX_SCORE, 10, &mut 0, 1.0), ulps=2);

        // minimizer
        assert_approx_eq!(f32, -4.75, -ff(&mut game, MAX_SCORE, 10, &mut 0, -1.0), ulps=2);

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
        assert_approx_eq!(f32, -4.5125, ff(&mut game, MAX_SCORE, 10, &mut 0, 1.0), ulps=2);

        // minimizer
        assert_approx_eq!(f32, 9.025, -ff(&mut game, MAX_SCORE, 10, &mut 0, -1.0), ulps=2);

        assert_approx_eq!(f32, -4.75, maximize(&mut game, 10, false).unwrap().score);
        assert_approx_eq!(f32, 9.5, minimize(&mut game, 10, false).unwrap().score);
    }

    #[test]
    fn case_3() {
        let mut arena = Arena::new();
        
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

        let ca = arena.new_node(0.);
        ca.append_value(10., &mut arena);
        ca.append_value(12., &mut arena);
        ca.append_value(3., &mut arena);

        let cb = arena.new_node(0.);
        cb.append_value(13., &mut arena);
        cb.append_value(random(), &mut arena);
        cb.append_value(random(), &mut arena);

        let c = arena.new_node(0.);
        c.append(ca, &mut arena);
        c.append(cb, &mut arena);

        let root = arena.new_node(0.);
        root.append(a, &mut arena);
        root.append(b, &mut arena);

        let mut game = Game {
            arena:arena,
            state:root,
        };

        assert_approx_eq!(f32, 10.2885, ff(&mut game, MAX_SCORE, 10, &mut 0, 1.0), ulps=2);
        let res = maximize(&mut game, 10, false).unwrap();
        println!("{:?}", res.ops_count);
        assert_approx_eq!(f32, 10.83, res.score);
    }
}
