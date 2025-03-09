use clap::ValueEnum;
use crate::{
    colour::{Hsl, Palette},
    maze::{Maze, NodeBuffer},
    state,
    Animation, Error, Settings 
};

mod aldous_broder;
mod dfs;
mod kruskal;
mod prim;
mod wilson;

/// State for the generate phase (currently none needed). 
pub struct Phase;

impl state::Phase for Phase {}

pub type State = state::State<Phase>;

/// Constructs a new global state for the generate phase. 
pub fn state(maze: Maze, settings: Settings) -> State {
    let age = NodeBuffer::new(&maze);
    let colours = {
        let base = Hsl {
            hue: 0.0, 
            saturation: 1.0, 
            lightness: 0.6, 
        };
        Palette::from_base(base).into_lut(&settings)
    };
    State {
        maze, 
        settings, 
        age, 
        visited_count: 0, 
        colours, 
        phase: Phase, 
    }
}

#[derive(ValueEnum, Clone, Copy, PartialEq, Eq)]
pub enum Generator {
    AldousBroder, 
    Dfs, 
    Kruskal, 
    Prim, 
    Wilson, 
}

impl Generator {
    pub fn run(self, state: &mut State) -> Result<(), Error> {
        match self {
            Generator::AldousBroder => aldous_broder::AldousBroder::run(state), 
            Generator::Dfs => dfs::Dfs::run(state), 
            Generator::Kruskal => kruskal::Kruskal::run(state), 
            Generator::Prim => prim::Prim::run(state), 
            Generator::Wilson => wilson::Wilson::run(state), 
        }
    }
}
