use crate::{
    maze::{Node, NodeBuffer}, 
    solve, 
    state, 
    Animation, 
    Signal, 
};

pub struct Walker {
    head: Node, 
}

pub struct Phase {
    start: Node,
    goal: Node, 
    parents: NodeBuffer<Option<Node>>, 
    on_path: NodeBuffer<bool>, 
}

impl state::Phase for Phase {
    fn special(&self, node: Node) -> bool {
        self.on_path[node] || [self.start, self.goal].contains(&node)
    }
}

pub type State = state::State<Phase>;

pub fn state(previous: solve::State) -> State {
    let solve::Phase{ start, goal, parents } = previous.phase;
    let phase = Phase {
        start, 
        goal, 
        parents, 
        on_path: NodeBuffer::new(&previous.maze), 
    };
    State {
        maze: previous.maze, 
        settings: previous.settings, 
        age: previous.age, 
        visited_count: previous.visited_count, 
        colours: previous.colours, 
        phase, 
    }
}

impl Animation for Walker {
    type Phase = Phase;

    fn new(state: &mut State) -> Self {
        Walker {
            head: state.goal, 
        }
    }

    fn step(&mut self, state: &mut State) -> Signal {
        let Some(head) = state.parents[self.head] else {
            return Signal::Done
        };
        state.on_path[head] = true;

        if head == state.start {
            return Signal::Done
        }
        self.head = head;
        Signal::Continue
    }
}
