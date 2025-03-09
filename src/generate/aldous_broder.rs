use crate::{
    maze::Node, 
    Animation, Signal
};
use super::{State, Phase};

/// Randomly traverse the grid and whenever we step out of the established maze, open the wall. 
pub struct AldousBroder {
    head: Node, 
}

impl Animation for AldousBroder {
    type Phase = Phase;
    
    fn new(state: &mut State) -> Self {
        let head = state.maze.random_node();
        AldousBroder{ head }
    }

    fn step(&mut self, state: &mut State) -> Signal {
        if state.all_visited() {
            return Signal::Done
        }
        state.visit(self.head);

        // take a step in a random direction from the head
        let edge = state.maze
            .neighbours(self.head)
            .choose();
        self.head = edge.to;

        // if we stepped out of the maze, open the wall
        if !state.is_visited(edge.to) {
            state.maze.open[edge] = true;
        }
        Signal::Continue
    }
}
