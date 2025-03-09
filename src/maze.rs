use std::{marker::PhantomData, ops::{Index, IndexMut}};
use arrayvec::ArrayVec;
use rand::{seq::{IteratorRandom, SliceRandom}, Rng};

/// Maze being constructed by [generators](crate::generate) and solved by [solvers](crate::solve).
///
/// A maze consists of a lattice of nodes connected by edges. Nodes are always considered open (traversable)
/// whereas edges can be either open or closed. 
pub struct Maze {
    /// Whether each edge in the maze is open. There are `(width - 1) * (height - 1)` edges. 
    pub open: EdgeBuffer<bool>, 
    /// Width in nodes. 
    pub width: usize, 
    /// Height in nodes. 
    pub height: usize, 
}

impl Maze {
    /// Constructs a maze from its dimensions. 
    pub fn new(width: u16, height: u16) -> Maze {
        let width = width as usize;
        let height = height as usize;
        debug_assert!(width != 0 && height != 0);

        Maze {
            width, 
            height, 
            open: EdgeBuffer::new_with_size(width, height), 
        }
    }

    /// Gets the node at given coordinates. 
    pub fn node(&self, x: usize, y: usize) -> Option<Node> {
        (x < self.width && y < self.height).then_some(Node(x, y))
    }

    /// Gets the edge relative to a node. 
    pub fn edge(&self, node: Node, direction: Direction) -> Option<Edge> {
        let Node(x, y) = node;
        let (nx, ny) = match direction {
            Direction::North => (x, y.wrapping_sub(1)), 
            Direction::South => (x, y.saturating_add(1)), 
            Direction::East  => (x.saturating_add(1), y), 
            Direction::West  => (x.wrapping_sub(1), y), 
        };
        self.node(nx, ny).map(|neighbour| Edge {
            from: node, 
            to: neighbour, 
            direction, 
        })
    }

    /// Chooses a random node in the maze. 
    pub fn random_node(&self) -> Node {
        let mut rng = rand::thread_rng();
        Node(
            rng.gen_range(0..self.width), 
            rng.gen_range(0..self.height), 
        )
    }

    /// Chooses a random node meeting some predicate, if there is one. 
    pub fn random_node_where(&self, predicate: impl Fn(Node) -> bool) -> Option<Node> {
        let mut rng = rand::thread_rng();
        self.nodes_iter()
            .filter(|&node| predicate(node))
            .choose(&mut rng)
    }

    /// Returns an iterator over all nodes. 
    pub fn nodes_iter(&self) -> impl Iterator<Item = Node> + use<> {
        let width = self.width;
        let height = self.height;
        (0..height).flat_map(move |y| (0..width).map(move |x| Node(x, y)))
    }

    /// Returns an iterator over all edges. 
    pub fn edges_iter(&self) -> impl Iterator<Item = Edge> {
        self.nodes_iter()
            .map(move |node| [
                self.edge(node, Direction::East), 
                self.edge(node, Direction::South), 
            ])
            .flatten()
            .flatten()
    }

    /// Returns a list of all neighbours to a node. 
    pub fn neighbours(&self, node: Node) -> Neighbours<true> {
        let neighbours = Direction::ALL
            .into_iter()
            .map(|d| self.edge(node, d))
            .flatten()
            .collect();
        Neighbours(neighbours)
    }

    /// Returns a list of all accessible neighbours to a node. 
    pub fn open_neighbours(&self, node: Node) -> Neighbours<false> {
        let neighbours = Direction::ALL
            .into_iter()
            .map(|d| self.edge(node, d))
            .flatten()
            .filter(|&e| self.open[e])
            .collect();
        Neighbours(neighbours)
    }

    /// Returns the top-left and bottom-right nodes. 
    pub fn bounds(&self) -> (Node, Node) {
        (Node(0, 0), Node(self.width - 1, self.height - 1))
    }
}

/// Direction relative to a [`Node`]. 
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Direction {
    North, 
    South, 
    East, 
    West, 
}

impl Direction {
    pub const ALL: [Direction; 4] = [
        Direction::North, 
        Direction::East, 
        Direction::South, 
        Direction::West, 
    ];

    pub fn clockwise(self) -> Direction {
        match self {
            Direction::North => Direction::East, 
            Direction::South => Direction::West, 
            Direction::East => Direction::South, 
            Direction::West => Direction::North, 
        }
    }

    pub fn anti_clockwise(self) -> Direction {
        self.clockwise().reverse()
    }

    pub fn reverse(self) -> Direction {
        match self {
            Direction::North => Direction::South, 
            Direction::South => Direction::North, 
            Direction::East => Direction::West, 
            Direction::West => Direction::East, 
        }
    }
}

/// List of neighbours to a [`Node`].
///
/// The type state `NON_EMPTY` states whether the list is known to be non-empty, which allows us to guarantee
/// correct unwrapping for methods like [`Neighbours::choose`]. 
pub struct Neighbours<const NON_EMPTY: bool>(ArrayVec<Edge, 4>);

impl<const NON_EMPTY: bool> Neighbours<NON_EMPTY> {
    /// Removes all neighbours not meeting the predicate. 
    pub fn filter(mut self, predicate: impl Fn(Node) -> bool) -> Neighbours<false> {
        self.0.retain(|e| predicate(e.to));
        Neighbours(self.0)
    }

    /// Gets the number of neighbours. 
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl Neighbours<true> {
    /// Chooses a random neighbour from the non-empty list. 
    pub fn choose(&self) -> Edge {
        self.0
            .as_slice()
            .choose(&mut rand::thread_rng())
            .copied()
            .unwrap()
    }
}

impl Neighbours<false> {
    /// Chooses a random (possibly filtered) neighbour, if one exists. 
    pub fn choose(&self) -> Option<Edge> {
        self.0
            .as_slice()
            .choose(&mut rand::thread_rng())
            .copied()
    }
}

impl<const NON_EMPTY: bool> IntoIterator for Neighbours<NON_EMPTY> {
    type Item = Edge;
    type IntoIter = <ArrayVec<Edge, 4> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// Generalisation over ways to index into a maze (e.g., nodes or edges). 
pub trait MazeIndex {
    /// The number of elements that can be indexed. 
    fn bound(maze_width: usize, maze_height: usize) -> usize;
    /// Iterator over all indices. 
    fn iter(maze: &Maze) -> impl Iterator<Item = Self>
        where Self: Sized;
    /// Normalises the index to a linear integer, which may be used to index an array. 
    fn normalise(&self, maze_width: usize) -> usize;
}

/// A node of the maze lattice. May be used to index a [`NodeBuffer`]. 
///
/// When constructed by [`Maze`], this is guaranteed to be in bounds. Despite this, the fields are public
/// (allowing for arbitrary construction) to simplify logic elsewhere. 
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Node(pub usize, pub usize);

impl Node {
    /// The manhattan distance between two nodes. 
    pub fn manhattan(self, other: Node) -> usize {
        usize::abs_diff(self.0, other.0) + usize::abs_diff(self.1, other.1)
    }
}

impl MazeIndex for Node {
    fn bound(maze_width: usize, maze_height: usize) -> usize {
        maze_width * maze_height
    }

    fn iter(maze: &Maze) -> impl Iterator<Item = Self> {
        maze.nodes_iter()
    }
    
    fn normalise(&self, maze_width: usize) -> usize {
        let Node(x, y) = self;
        x + y * maze_width
    }
}

/// An edge of the maze lattice. May be used to index an [`EdgeBuffer`]. 
///
/// When constructed by [`Maze`], this is guaranteed to be in bounds. Despite this, the fields are public
/// (allowing for arbitrary construction) to simplify logic elsewhere. 
#[derive(Clone, Copy, Debug, Hash, Eq)]
pub struct Edge {
    pub from: Node, 
    pub to: Node, 
    pub direction: Direction, 
}

impl Edge {
    /// Returns an edge pointing from the given node to itself. 
    pub fn identity(node: Node) -> Edge {
        Edge {
            from: node, 
            to: node, 
            direction: Direction::North, 
        }
    }

    pub fn reverse(self) -> Edge {
        Edge {
            from: self.to,
            to: self.from,
            direction: self.direction.reverse(), 
        }
    }
}

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        (self.from, self.to) == (other.from, other.to) ||
        (self.from, self.to) == (other.to, other.from)
    }
}

impl MazeIndex for Edge {
    fn bound(maze_width: usize, maze_height: usize) -> usize {
        2 * maze_width * maze_height
    }

    fn iter(maze: &Maze) -> impl Iterator<Item = Self> {
        maze.edges_iter()
    }

    fn normalise(&self, maze_width: usize) -> usize {
        let Node(x, y) = self.from;
        let (x, y, z) = match self.direction {
            Direction::North => (x,   y-1, 0), 
            Direction::South => (x,   y,   0), 
            Direction::East  => (x,   y,   1), 
            Direction::West  => (x-1, y,   1), 
        };
        let owner = Node(x, y);
        2 * owner.normalise(maze_width) + z
    }
}

/// A buffer indexable by any [`MazeIndex`] storing arbitrary data. 
///
/// Internally, this uses [`MazeIndex::normalise`] to index a linear array, ensuring efficient data layout. 
pub struct Buffer<T, U> {
    /// Data being stored. 
    data: Vec<U>, 
    /// Maze width, used by [`MazeIndex::normalise`]. 
    width: usize, 
    /// Phandom data for the index type. 
    _phantom: PhantomData<T>, 
}

impl<T: MazeIndex, U> Buffer<T, U> {
    /// Constructs a buffer with default values for each element. 
    pub fn new(maze: &Maze) -> Self
    where
        U: Clone + Default
    {
        Self::new_with_values(maze, U::default())
    }

    /// Constructs a buffer with given value cloned for each element. 
    pub fn new_with_values(maze: &Maze, value: U) -> Self
    where
        U: Clone
    {
        let size = T::bound(maze.width, maze.height);
        Self {
            data: vec![value; size], 
            width: maze.width, 
            _phantom: PhantomData, 
        }
    }

    /// Constructs a buffer with a value given by a function over the index for each element. 
    pub fn new_from_function(maze: &Maze, op: impl FnMut(T) -> U) -> Self {
        Self {
            data: T::iter(maze).map(op).collect(), 
            width: maze.width,
            _phantom: PhantomData, 
        }
    }

    /// Constructs a buffer with given maze dimensions. This is only used by [`Maze::new`] since we don't
    /// have a [`Maze`] instance yet. 
    fn new_with_size(width: usize, height: usize) -> Self
    where
        U: Default + Clone
    {
        let size = T::bound(width, height);
        Self {
            data: vec![U::default();  size], 
            width,
            _phantom: PhantomData, 
        }
    }

    /// Returns an iterator over the value for each element. 
    pub fn iter(&self) -> impl Iterator<Item = &U> {
        self.data.iter()
    }

    /// Returns a mutable iterator over the value for each element. 
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut U> {
        self.data.iter_mut()
    }

    /// Clones given value into each element. 
    pub fn fill(&mut self, value: U) where
        U: Clone
    {
        for x in self.data.iter_mut() {
            *x = value.clone();
        }
    }
}

impl<T: MazeIndex, U> Index<T> for Buffer<T, U> {
    type Output = U;

    fn index(&self, index: T) -> &U {
        let index = index.normalise(self.width);
        &self.data[index]
    }
}

impl<T: MazeIndex, U> IndexMut<T> for Buffer<T, U> {
    fn index_mut(&mut self, index: T) -> &mut U {
        let index = index.normalise(self.width);
        &mut self.data[index]
    }
}

/// Buffer indexable by [`Node`]. 
pub type NodeBuffer<T> = Buffer<Node, T>;
/// Buffer indexable by [`Edge`]. 
pub type EdgeBuffer<T> = Buffer<Edge, T>;
