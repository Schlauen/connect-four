/// Implemented methods should in general not call each other.
/// State should be persisted and invalidated if necessary
pub trait Environment<T> {
    /// Evaluate the current environment state
    fn evaluate(&mut self) -> i8;
    
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

pub fn minimize<T>(env:&mut impl Environment<T>, level:u8) -> Option<T> {
    if level == 0 || env.is_finished() {
        return Option::None;
    }

    let mut min_value = i8::MAX;
    let mut best_action = Option::None;
    let a = i8::MIN;

    for action in env.actions() {
        env.apply(&action);
        let value = max_(env, a, min_value, level - 1);
        env.revert(&action);

        if value < min_value {
            min_value = value;
            best_action = Option::Some(action);
            if min_value <= a {
                break;
            }
        }
    }
    best_action
} 

pub fn maximize<T>(env:&mut impl Environment<T>, level:u8) -> Option<T> {
    if level == 0 || env.is_finished() {
        return Option::None;
    }

    let mut max_value = i8::MIN;
    let mut best_action = Option::None;
    let b = i8::MAX;
    
    for action in env.actions() {
        env.apply(&action);
        let value = min_(env, max_value, b, level - 1);
        env.revert(&action);

        if value > max_value {
            max_value = value;
            best_action = Option::Some(action);
            if max_value >= b {
                break;
            }
        }
    }
    best_action
} 


fn min_<T>(env:&mut impl Environment<T>, a:i8, b:i8, level:u8) -> i8 {
    if level == 0 || env.is_finished() {
        return env.evaluate();
    }

    env.swap_players();

    // min can certainly achieve this value or better (less)
    let mut min_value = b;
    
    for action in env.actions() {
        env.apply(&action);
        let value = max_(env, a, min_value, level - 1);
        env.revert(&action);

        if value < min_value {
            min_value = value; 
            if min_value <= a {
                break;
            }
        }
    }

    env.swap_players();
    min_value
}

fn max_<T>(env:&mut impl Environment<T>, a:i8, b:i8, level:u8) -> i8 {
    if level == 0 || env.is_finished() {
        return env.evaluate();
    }

    env.swap_players();
    
    // max can certainly achieve this value or better (more)
    let mut max_value = a;
    
    for action in env.actions() {
        env.apply(&action);
        let value = min_(env, max_value, b, level - 1);
        env.revert(&action);

        if value > max_value {
            max_value = value;
            if max_value >= b {
                break;
            }
        }
    }

    env.swap_players();
    max_value
}

#[cfg(test)]
mod tests {
    use indextree::{Arena, NodeId};
    use rand::prelude::*;
    use super::*;

    struct Game {
        arena:Arena<i8>,
        state:NodeId,
    }
    
    impl Environment<usize> for Game {
        fn evaluate(&mut self) -> i8 {
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
        
        fn swap_players(&mut self) {
            todo!()
        }
    }

    #[test]
    fn simple_case() {      
        let mut arena = Arena::new();

        let root = arena.new_node(0);
        root.append_value(10, &mut arena);
        root.append_value(-5, &mut arena);

        let mut game = Game {
            arena:arena,
            state:root,
        };

        assert_eq!(10, max_(&mut game, i8::MIN, i8::MAX, 10));
        assert_eq!(-5, min_(&mut game, i8::MIN, i8::MAX, 10));

        assert_eq!(Some(0), maximize(&mut game, 10));
        assert_eq!(Some(1), minimize(&mut game, 10));
    }

    #[test]
    fn case_2() {
        let mut arena = Arena::new();
        
        let aa = arena.new_node(0);
        aa.append_value(10, &mut arena);
        aa.append_value(-5, &mut arena);
        aa.append_value(3, &mut arena);

        let ab = arena.new_node(0);
        ab.append_value(-6, &mut arena);
        ab.append_value(15, &mut arena);
        ab.append_value(random(), &mut arena);

        let a = arena.new_node(0);
        a.append(aa, &mut arena);
        a.append(ab, &mut arena);

        let mut game = Game {
            arena:arena,
            state:a,
        };
        assert_eq!(10, min_(&mut game, i8::MIN, i8::MAX, 10));
        assert_eq!(Some(0), minimize(&mut game, 10));
    }

    #[test]
    fn case_3() {
        let mut arena = Arena::new();
        
        let aa = arena.new_node(0);
        aa.append_value(10, &mut arena);
        aa.append_value(-5, &mut arena);
        aa.append_value(3, &mut arena);

        let ab = arena.new_node(0);
        ab.append_value(-6, &mut arena);
        ab.append_value(15, &mut arena);
        ab.append_value(random(), &mut arena);

        let a = arena.new_node(0);
        a.append(aa, &mut arena);
        a.append(ab, &mut arena);

        let ba = arena.new_node(0);
        ba.append_value(10, &mut arena);
        ba.append_value(12, &mut arena);
        ba.append_value(3, &mut arena);

        let bb = arena.new_node(0);
        bb.append_value(13, &mut arena);
        bb.append_value(random(), &mut arena);
        bb.append_value(random(), &mut arena);

        let b = arena.new_node(0);
        b.append(ba, &mut arena);
        b.append(bb, &mut arena);

        let ca = arena.new_node(0);
        ca.append_value(10, &mut arena);
        ca.append_value(12, &mut arena);
        ca.append_value(3, &mut arena);

        let cb = arena.new_node(0);
        cb.append_value(13, &mut arena);
        cb.append_value(random(), &mut arena);
        cb.append_value(random(), &mut arena);

        let c = arena.new_node(0);
        c.append(ca, &mut arena);
        c.append(cb, &mut arena);

        let root = arena.new_node(0);
        root.append(a, &mut arena);
        root.append(b, &mut arena);

        let mut game = Game {
            arena:arena,
            state:root,
        };

        assert_eq!(12, max_(&mut game, i8::MIN, i8::MAX, 10));
        assert_eq!(Some(1), maximize(&mut game, 10));
    }
}
