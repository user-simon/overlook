use crate::{
    maze::{Edge, NodeBuffer}, 
    Animation, Signal, 
};
use super::{State, Phase};

/// Perform a loop-erased random walk from a random unvisited node until we find the established maze. 
pub struct Wilson {
    /// Loop-erased path currently being walked. 
    path: Vec<Edge>, 
    /// Path indices of successor nodes. When a loop is detected, this is the first index to remove. 
    path_indices: NodeBuffer<Option<usize>>, 
}

impl Animation for Wilson {
    type Phase = Phase;
    
    fn new(state: &mut State) -> Self {
        let goal = state.maze.random_node();
        state.visit(goal);
        
        Wilson {
            path: Vec::default(), 
            path_indices: NodeBuffer::new(&state.maze), 
        }
    }

    fn step(&mut self, state: &mut State) -> Signal {
        // get last node in path or begin a new path if it is empty
        let head = match self.path.last() {
            Some(last) => last.to, 
            None => {
                let first = state.maze.random_node_where(|n| !state.is_visited(n));
                let Some(first) = first else {
                    return Signal::Done
                };
                self.path.push(Edge::identity(first));
                first
            }
        };
        state.visit(head);
        self.path_indices[head] = Some(self.path.len());

        // choose next node in the path
        let edge = state.maze
            .neighbours(head)
            .choose();
        let next = edge.to;

        match (self.path_indices[next], state.is_visited(next)) {
            // the next node is already in the path => we have a loop and need to erase it
            (Some(loop_start), _) => {
                for erased in self.path.drain(loop_start..) {
                    self.path_indices[erased.to] = None;
                    state.maze.open[erased] = false;
                    state.unvisit(erased.to);
                }
            }
            // the next node is not on the path but is visited => we have found the established maze and can
            // finalise the path
            (None, true) => {
                state.maze.open[edge] = true;
                self.path_indices.fill(None);
                self.path.clear();
            }
            // otherwise, we continue the path
            (None, false) => {
                state.maze.open[edge] = true;
                self.path.push(edge);
            } 
        }
        Signal::Continue
    }

    fn timescale(&self) -> u32 {
        125
    }
}
