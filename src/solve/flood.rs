use std::collections::VecDeque;
use crate::{
    maze::Edge, 
    Animation, Signal, 
};
use super::{State, Phase};

/// Search the maze breadth-first from start to goal. 
///
/// This implementation is somewhat complex in order for it to animate smoothly (it traverses all queued
/// nodes in a step, instead of just one). This is achieved using a double-buffer of queued nodes; queue A
/// for nodes to be visited this timestep, and queue B for nodes to be visited next timestep. 
pub struct Flood {
    /// Nodes to be visited this timestep. 
    queue_a: VecDeque<Edge>, 
    /// Nodes to be visited next timestep. 
    queue_b: VecDeque<Edge>, 
}

impl Animation for Flood {
    type Phase = Phase;

    fn new(state: &mut State) -> Self {
        Flood {
            queue_a: VecDeque::from([Edge::identity(state.start)]), 
            queue_b: VecDeque::new(), 
        }
    }

    fn step(&mut self, state: &mut State) -> Signal {
        let Some(head) = self.queue_a.pop_front() else {
            return match self.queue_b.is_empty() {
                true => Signal::Done,
                false => {
                    std::mem::swap(&mut self.queue_a, &mut self.queue_b);
                    Signal::Continue
                }
            }
        };

        state.visit(head.to);
        state.parents[head.to] = Some(head.from);

        if head.to == state.goal {
            return Signal::Done
        }

        let open_neighbours = state.maze
            .open_neighbours(head.to)
            .into_iter()
            .filter(|&e| !state.is_visited(e.to));
        self.queue_b.extend(open_neighbours);

        self.step(state)
    }

    fn timescale(&self) -> u32 {
        75
    }
}
