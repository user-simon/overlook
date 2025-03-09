![banner img](img/banner.png)

# ⛕ Overlook

An animated visualiser of different maze generation and solving algorithms, running the terminal. 

The following generation algorithms are implemented:
- Aldous-broder
- Randomised depth-first search
- Randomised Kruskal's algorithm
- Randomised Prim's algorithm
- Wilson's algorithm

And the following solving algorithms are implemented:
- A★
- Flood fill
- Random mouse
- Right-hand rule


# 📌 Examples[^1]

Generate with Kruskal's algorithm and solve with flood fill:
```
$ overlook --generator kruskal --solver flood
```

https://github.com/user-attachments/assets/ea4b75e2-c3dd-4f59-b7e3-d7b6e8893d48

Generate with randomised depth-first search and solve with the right-hand rule: 
```
$ overlook --generator dfs --solver right-hand
```

https://github.com/user-attachments/assets/750e35f5-8370-43e1-aa76-1a0455e81bb3

Generate with Wilson's algorithm and solve with A★:
```
$ overlook --generator wilson --solver a-star
```

https://github.com/user-attachments/assets/4032ab4c-dcaa-4e74-8386-35263401101c


# 🖥️ Usage

For the best experience, use a terminal emulator with [true color support](https://gist.github.com/kurahaupo/6ce0eaefe5e730841f03cb82b061daa2#now-supporting-true-color). Terminal emulators that only support ANSI escape codes may be used with the `--ansi` flag. 

```
Usage: overlook [OPTIONS] --generator <GENERATOR> --solver <SOLVER>

Options:
    -w, --width <WIDTH>          Maze width in nodes
    -h, --height <HEIGHT>        Maze height in nodes
    -g, --generator <GENERATOR>  [possible values: aldous-broder, dfs, kruskal, prim, wilson]
    -s, --solver <SOLVER>        [possible values: a-star, flood, mouse, right-hand]
    -d, --delay <DELAY>          Animation timestep [default: 100ms]
    -a, --ansi                   Renders the maze using only standard ANSI colours
        --help                   Print help
```


# 🛠️ Write-up

## Architecture

At the core of `overlook` is the _animation_ trait. As its name suggests, it defines an animation running in realtime using a discrete `step` function,[^2] which has mutable access to our _state_. The state contains e.g. the maze, whether nodes have yet been visited, and global settings derived from the CLI. 

Animations performing the same task (e.g. generating or solving the maze) will generally need the same kinds of state. Instead of repeating these for each animation, we define groupings of animations with similar state-needs as _phases_. The following phases are defined (in order of execution): 
- Generate: generate the maze
- Solve: solve the maze from start to goal
- Walk: walk the maze from goal back to start using the parent LUT


## Maze

The maze is represented as a lattice of nodes connected by edges, which may be open to signify a passage, or closed to signify a wall. Although this is the representation, this lattice is never stored in memory, and traversed through (encapsulated) coordinate arithmetic. 

The fundamental data structures for interfacing with the maze is _node buffers_ and _edge buffers_. They are one-dimensional arrays indexable by a node (as defined by its $xy$ coordinates) or an edge (as defined by a "from" node, "to" node, and a direction), respectively. It uses the dimensions of the maze to flatten either index to an integer, allowing efficient data layout. 

Nodes indices are trivially flattened according to $x + y * \text{width}$. To flatten edge indices, we ascribe each edge to a node; the node to the west for horizontal edges, and the node to the north for vertical edges. To flatten, we then get the flattened index of the ascribed node, multiply it by two (since each node owns two edges), and add $0$ for horizontal edges (the edge east of the node) and $1$ for vertical edges (the edge south of the node). 

Inside the maze we keep an edge buffer of booleans indicating whether each edge is open. Other node and edge buffers are used by the implementations of individual animations. This allows us to spare precious nanoseconds in this _extremely_ (very) time-sensitive program by not using hash-tables as lookups. 


## Colours

Each phase defines a colour scheme to be used when rendering the maze. The colour scheme contains colours for:
- The youngest node
- The oldest node
- Unvisited nodes
- Special nodes

When rendering a node, we then linearly interpolate between the colour of the youngest and oldest node using the node's age ($0-255$) as factor. This interpolation looks terrible in the RGB colour space, and we'd like to randomise the hues for variety, so we need a different colour space. HSL meets the requirements, but suffers from inconsistent perceived luminosities at different hues (e.g., pure yellow being perceived as much "brighter" than pure blue), which makes it difficult to design a colour scheme that looks good regardless of the hue. Though no colour space is perfect, we find that Okhsl does an ok job and use it for our colours. 

Since we can't render Okhsl colours directly, we first need to convert them to RGB, which can be slow. To combat this, we precompute the conversions for all age values $0-255$, yielding a LUT of RGB values. Using this LUT as the interface when rendering also allows us to easily define arbitrary colour schemes, such as only using standard ANSI colours (exposed via `--ansi`). 

Edges will also need to be rendered if they are open, which is done using the colour corresponding to the oldest of two nodes it connects (if we instead used the youngest, we would find that the offshoots of a corridor being traversed would light up). 


[^1]: These MP4s are so jank but all the animations are _way_ too large for asciinema's servers. 
[^2]: I never knew my real function. 
