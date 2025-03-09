use std::{cmp::Reverse, collections::BinaryHeap};
use crate::{
    maze::{Node, NodeBuffer}, 
    Animation, Signal, 
};
use super::{State, Phase};

/// Search guided by Euclidian distance. 
/// 
/// This implementation is simplified from canonical ones since we can assume our mazes are free from loops. 
pub struct AStar {
    /// Min-heap of `f`-scores (the [`Reverse`] makes it min and not max). 
    heap: BinaryHeap<(Reverse<usize>, Node)>, 
    /// `g`-scores of all nodes. 
	g_score: NodeBuffer<usize>, 
}

impl Animation for AStar {
    type Phase = Phase;

    fn new(state: &mut State) -> Self {
        let mut g_score = NodeBuffer::new_with_values(&state.maze, usize::MAX); 
        g_score[state.start] = 0;
        
        AStar {
            heap: BinaryHeap::from([entry(0, state.start, state)]), 
            g_score, 
        }
    }

    fn step(&mut self, state: &mut State) -> Signal {
        let Some((_, head)) = self.heap.pop() else {
            return Signal::Done
        };
        state.visit(head);

        if head == state.goal {
            return Signal::Done
        }

        let neighbours = state.maze
            .open_neighbours(head)
            .filter(|n| !state.is_visited(n));

        for edge in neighbours {
            let neighbour = edge.to;
            state.parents[neighbour] = Some(head);

            let g_score = self.g_score[head] + 1;
            self.g_score[neighbour] = g_score;
            self.heap.push(entry(g_score, neighbour, state));
        }
        Signal::Continue
    }

}

fn entry(g_score: usize, node: Node, state: &mut State) -> (Reverse<usize>, Node) {
    let f_score = g_score + Node::manhattan(node, state.goal);
    (Reverse(f_score), node)
}
