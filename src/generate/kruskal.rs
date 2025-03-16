use rand::seq::SliceRandom;
use crate::{
    maze::{Edge, Maze, Node, NodeBuffer}, 
    Animation, Signal
};
use super::{State, Phase};

/// Iteratively adjoins trees of nodes, with each node initially being given its own tree, until all nodes
/// belong to the same tree. 
pub struct Kruskal {
    /// A queue of all edges in a random order. 
    queue: Vec<Edge>, 
    /// Stores the parent of each node. This is used to compute root nodes (nodes that are their own parents)
    /// which serves as set identifiers. 
    parents: NodeBuffer<Node>, 
}

impl Kruskal {
    /// Finds the root of a node, and sets it as the direct parent of the node and all nodes inbetween for
    /// quick future lookup. 
    fn find_root(&mut self, maze: &Maze, node: Node) -> Node {
        let parent = self.parents[node];

        match node == parent {
            true => node, 
            false => {
                let root = self.find_root(maze, parent);
                self.parents[node] = root;
                root
            }
        }
    }
}

impl Animation for Kruskal {
    type Phase = Phase;

    fn new(state: &mut State) -> Self {
        let queue = {
            let mut walls: Vec<Edge> = state.maze.edges_iter().collect();
            walls.shuffle(&mut rand::thread_rng());
            walls
        };
        let parents = NodeBuffer::new_from_function(&state.maze, std::convert::identity);
        Kruskal{ queue, parents }
    }

    fn step(&mut self, state: &mut State) -> Signal {
        let Some(edge) = self.queue.pop() else {
            return Signal::Done
        };

        // open the edge only if the two nodes aren't in the same set (have the same root node) as we would
        // otherwise introduce a loop
        let root_a = self.find_root(&state.maze, edge.from);
        let root_b = self.find_root(&state.maze, edge.to);

        if root_a != root_b {
            state.visit(edge.from);
            state.visit(edge.to);

            self.parents[root_a] = root_b;
            state.maze.open[edge] = true;
            Signal::Continue
        } else { // otherwise, keep searching for an openable wall
            self.step(state)
        }
    }

    fn timescale(&self) -> u32 {
        150
    }
}
