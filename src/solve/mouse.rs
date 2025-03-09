use crate::{maze::Node, Animation, Signal};
use super::{State, Phase};

/// Perform a random walk in the maze until the goal is found. 
pub struct Mouse {
	head: Node, 
}

impl Animation for Mouse {
    type Phase = Phase;

    fn new(state: &mut State) -> Self {
    	Mouse {
    		head: state.start, 
    	}
    }

    fn step(&mut self, state: &mut State) -> Signal {
		state.visit(self.head);

    	if self.head == state.goal {
    		return Signal::Done
    	}
    	
		let edge = state.maze
			.open_neighbours(self.head)
			.choose()
			.expect("There are no isolated nodes");
		let next = edge.to;

		// we have to take care not to introduce a loop
		state.parents[next].get_or_insert(self.head);
		self.head = next;
		Signal::Continue
    }
}
