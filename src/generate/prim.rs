use rand::Rng;
use crate::{
    maze::Edge, 
    Animation, Signal
};
use super::{State, Phase};

pub struct Prim {
    queue: Vec<Edge>, 
}

impl Animation for Prim {
    type Phase = Phase;
    
    fn new(state: &mut State) -> Self {
        let start = state.maze.random_node();
        let queue = state.maze.neighbours(start).into_iter().collect();
        state.visit(start);
        Prim{ queue }
    }

    fn step(&mut self, state: &mut State) -> Signal {
        if self.queue.is_empty() {
            return Signal::Done
        }
        let index = rand::thread_rng().gen_range(0..self.queue.len());
        let edge = self.queue.swap_remove(index);
        let unvisited = match (state.is_visited(edge.from), state.is_visited(edge.to)) {
            (true, false) => Some(edge.to), 
            (false, true) => Some(edge.from), 
            _ => None, 
        };

        state.visit(edge.from);
        state.visit(edge.to);

        if let Some(unvisited) = unvisited {
            self.queue.extend(state.maze.neighbours(unvisited));
            state.maze.open[edge] = true;
        }
        Signal::Continue
    }
}

