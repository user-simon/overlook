use std::{fmt, ops::{Deref, DerefMut}};
use crossterm::style::{StyledContent, Stylize};
use crate::{
    colour::Lut, 
    maze::{Direction, Maze, Node, NodeBuffer}, 
    Settings, 
};

/// Global state of the program. 
pub struct State<T> {
    /// The maze being operated upon. 
    pub maze: Maze, 
    /// Settings used. 
    pub settings: Settings, 
    /// The age of each visited [`Node`]. This is set by [`State::visit`] and incremented by [`State::step`]. 
    pub age: NodeBuffer<Option<u8>>, 
    /// Current number of visited nodes. 
    pub visited_count: usize, 
    /// Node colour lookup. 
    pub colours: Lut, 
    /// State specific to each [`Phase`]. 
    pub phase: T, 
}

impl<T: Phase> State<T> {
    /// Marks the given node as visited, with custom age. 
    pub fn set_age(&mut self, node: Node, age: u8) {
        if let None = self.age[node].replace(age) {
            self.visited_count += 1;
        }
    }
    
    /// Marks the given node as visited, with age zero. 
    pub fn visit(&mut self, node: Node) {
        self.set_age(node, 0);
    }

    /// Unmarks the given node as visited. 
    pub fn unvisit(&mut self, node: Node) {
        if let Some(_) = self.age[node].take() {
            self.visited_count -= 1;
        }
    }

    /// Returns whether the given node is visited. 
    pub fn is_visited(&self, node: Node) -> bool {
        self.age[node].is_some()
    }

    /// Whether all nodes of the maze have been visited. 
    pub fn all_visited(&self) -> bool {
        self.visited_count == self.maze.width * self.maze.height
    }

    /// Increments the ages of all visited nodes. 
    ///
    /// This is a bit of a hack given that it iterates over all nodes each timestep, but I can't figure out a
    /// better way to handle aging cells without overflow issues. 
    pub fn step(&mut self) {
        for age in self.age.iter_mut().flatten() {
            *age = age.saturating_add(1);
        }
    }

    fn format_coloured(&self, age: Option<u8>, special: bool) -> StyledContent<&str> {
        let colour = special
            .then_some(self.colours.special)
            .unwrap_or_else(|| self.colours.sample(age));
        "  ".on(colour)
    }
}

impl<T> Deref for State<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.phase
    }
}

impl<T> DerefMut for State<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.phase
    }
}

impl<T: Phase> fmt::Display for State<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        const EMPTY: &'static str = "  ";

        let State{ maze, phase, ..} = &self;
        let format_edge = |node, direction| maze.edge(node, direction)
            .map(|e| match maze.open[e] {
                true => self.format_coloured(
                    // if either node is unvisited, draw as unvisited. otherwise, draw oldest age
                    Option::zip(self.age[e.from], self.age[e.to]).map(|(a, b)| u8::max(a, b)), 
                    // draw special if both nodes are special
                    self.special(e.from) && self.special(e.to),
                ), 
                false => EMPTY.stylize(), 
            })
            .unwrap_or("".stylize());

        for y in 0..maze.height {
            // draw first row
            for x in 0..maze.width {
                let node = Node(x, y);
                let east_str = format_edge(node, Direction::East);
                let node_str = self.format_coloured(self.age[node], phase.special(node));
                write!(f, "{node_str}{east_str}")?;
            }

            if y == maze.height - 1 {
                continue
            }
            write!(f, "\n\r")?;

            // draw second row
            for x in 0..maze.width {
                let node = Node(x, y);
                let south_str = format_edge(node, Direction::South);
                write!(f, "{south_str}{EMPTY}")?;
            }
            write!(f, "\n\r")?;
        }
        fmt::Result::Ok(())
    }
}

/// Generalisation over different application phases. 
pub trait Phase {
    /// Whether a node should be drawn as "special". 
    fn special(&self, _node: Node) -> bool {
        false
    }
}
