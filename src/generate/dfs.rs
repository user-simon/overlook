use crate::{
    maze::Node, 
    Animation, Signal
};
use super::{State, Phase};

/// Perform a depth-first search to traverse the entire grid and randomly choose a direction at each
/// intersection. 
pub struct Dfs {
    stack: Vec<Node>, 
}

impl Animation for Dfs {
    type Phase = Phase;
    
    fn new(state: &mut State) -> Self {
        Dfs {
            stack: vec![state.maze.random_node()], 
        }
    }

    fn step(&mut self, state: &mut State) -> Signal {
        let Some(head) = self.stack.pop() else {
            return Signal::Done
        };
        state.visit(head);

        // take a step toward a random unvisited neighbour
        let neighbour = state.maze
            .neighbours(head)
            .filter(|n| !state.is_visited(n))
            .choose();

        // if there is one, add it to the stack and open the wall between them
        if let Some(edge) = neighbour {
            self.stack.push(head);
            self.stack.push(edge.to);
            state.maze.open[edge] = true;
        }
        Signal::Continue
    }
}
