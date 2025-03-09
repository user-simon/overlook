use std::collections::VecDeque;
use clap::ValueEnum;
use crate::{
    colour::Palette, 
    generate, 
    maze::{Edge, Maze, Node, NodeBuffer}, 
    state, 
    Animation, Error, 
};

mod a_star;
mod flood;
mod mouse;
mod right_hand;

/// State for the solve phase. 
pub struct Phase {
    /// Node being searched from. 
    pub start: Node, 
    /// Node being searched to. 
    pub goal: Node, 
    /// The parent of each visited node. 
    pub parents: NodeBuffer<Option<Node>>, 
}

impl state::Phase for Phase {
    fn special(&self, node: Node) -> bool {
        [self.start, self.goal].contains(&node)
    }
}

pub type State = state::State<Phase>;

pub fn state(previous: generate::State) -> State {
    let age = NodeBuffer::new(&previous.maze);
    let parents = NodeBuffer::new(&previous.maze);
    let gradient = {
        let base = previous.colours.palette.young;
        let unvisited = base
            .with_s(0.7)
            .with_l(0.17);
        let special = base
            .shift_h(60.0)
            .with_l(0.7);
        Palette::from_base(base)
            .with_unvisited(unvisited)
            .with_special(special)
            .into_lut(&previous.settings)
    };
    let (top_left, bottom_right) = previous.maze.bounds();
    let start = find_dead_end(top_left, &previous.maze);
    let goal = find_dead_end(bottom_right, &previous.maze);

    State {
        maze: previous.maze, 
        settings: previous.settings, 
        age, 
        visited_count: 0, 
        colours: gradient, 
        phase: Phase {
            start, 
            goal, 
            parents, 
        }, 
    }
}

#[derive(ValueEnum, Clone, Copy, PartialEq, Eq)]
pub enum Solver {
    AStar, 
    Flood, 
    Mouse, 
    RightHand, 
}

impl Solver {
    pub fn run(self, state: &mut State) -> Result<(), Error> {
        match self {
            Solver::AStar => a_star::AStar::run(state), 
            Solver::Flood => flood::Flood::run(state), 
            Solver::Mouse => mouse::Mouse::run(state), 
            Solver::RightHand => right_hand::RightHand::run(state), 
        }
    }
}

fn find_dead_end(from: Node, maze: &Maze) -> Node {
    let mut queue = VecDeque::from([Edge::identity(from)]);

    while let Some(head) = queue.pop_front() {
        let neighbours = maze.open_neighbours(head.to);

        if neighbours.len() == 1 {
            return head.to
        } else {
            queue.extend(neighbours.into_iter());
        }
    }
    return from
}
