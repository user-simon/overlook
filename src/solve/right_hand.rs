use crate::{
    maze::{Direction, Node}, 
    Animation, Signal
};
use super::{State, Phase};

pub struct RightHand {
    head: Node, 
    direction: Direction, 
}

impl Animation for RightHand {
    type Phase = Phase;

    fn new(state: &mut State) -> RightHand {
        RightHand {
            head: state.start,
            direction: Direction::North, 
        }
    }

    fn step(&mut self, state: &mut State) -> Signal {
        state.visit(self.head);

        if self.head == state.goal {
            return Signal::Done
        }

        let next = state.maze
            .edge(self.head, self.direction)
            .filter(|&e| state.maze.open[e]);

        match next {
            Some(e) => {
                self.head = e.to;
                self.direction = e.direction.clockwise();

                // we have to take care not to introduce a loop
                state.parents[e.to].get_or_insert(e.from);
                Signal::Continue
            }
            None => {
                self.direction = self.direction.anti_clockwise();
                self.step(state)
            }
        }
    }
}
