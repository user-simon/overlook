use std::{io, time::Duration};
use clap::{ArgAction, Parser};
use crossterm::{
    cursor::{Hide, MoveTo, Show}, 
    style::Print, 
    terminal::{EnterAlternateScreen, LeaveAlternateScreen}, 
};
use rand::Rng;
use walk::Walker;
use crate::{
    generate::Generator, 
    maze::Maze, 
    solve::Solver, 
    state::{Phase, State}, 
};

mod fade;
mod generate;
mod colour;
mod maze;
mod state;
mod solve;
mod walk;

/// Signals the algorithm runtime what to do after each timestep. 
pub enum Signal {
    Continue, 
    Done, 
}

pub enum Error {
    Io(io::Error),
    Break, 
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

/// Generalisation over different algorithms being animated upon a maze. 
pub trait Animation: Sized {
    type Phase: Phase;
    
    /// Constructs the algorithm from a given state. 
    fn new(state: &mut State<Self::Phase>) -> Self;
    /// Moves the animation one timestep forward. 
    fn step(&mut self, state: &mut State<Self::Phase>) -> Signal;
    /// Animation timescale working in conjunction with [`Settings::delay`]. 
    fn timescale(&self) -> u32 {
        100
    }
    
    /// Runs the animation until it signals to stop, printing the current state at each timestep. 
    fn run(state: &mut State<Self::Phase>) -> Result<(), Error> {
        let mut algorithm = Self::new(state);

        while let Signal::Continue = algorithm.step(state) {
            crossterm::execute!{
                io::stdout(), 
                MoveTo(0, 0), 
                Print(&state), 
            }?;
            state.step();

            let delay = 100 * state.settings.delay / algorithm.timescale();

            if crossterm::event::poll(delay)? {
                return Err(Error::Break)
            }
        }
        Ok(())
    }
}

/// Sets up the terminal environment. 
fn setup() -> io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!{
        std::io::stdout(), 
        EnterAlternateScreen, 
        Hide, 
    }?;
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        reset();
        prev(info);
    }));
    Ok(())
}

/// Resets the terminal environment. 
fn reset() {
    let _ = crossterm::execute!{
        std::io::stdout(), 
        LeaveAlternateScreen, 
        Show, 
    };
    let _ = crossterm::terminal::disable_raw_mode();
}

#[derive(Parser)]
#[clap(disable_help_flag = true)]
struct Cli {
    /// Maze width in nodes. 
    #[arg(long, short)]
    #[arg(value_parser = clap::value_parser!(u16).range(2..))]
    width: Option<u16>, 

    /// Maze height in nodes. 
    #[arg(long, short)]
    #[arg(value_parser = clap::value_parser!(u16).range(2..))]
    height: Option<u16>, 

    #[arg(long, short)]
    generator: Generator, 

    #[arg(long, short)]
    solver: Solver, 

    /// Animation timestep. 
    #[arg(long, short, default_value="100ms")]
    delay: humantime::Duration, 

    /// Renders the maze using only standard ANSI colours. 
    #[arg(long, short)]
    ansi: bool, 

    /// Print help. 
    #[arg(long, action=ArgAction::HelpLong)]
    help: Option<bool>, 
}

pub struct Settings {
    pub delay: Duration, 
    pub ansi: bool, 
    pub hue_shift: f64, 
}

fn main() {
    fn inner() -> Result<(), Error> {
        let cli = Cli::parse();

        setup()?;

        let terminal_size = crossterm::terminal::size()?;
        let width = cli.width.unwrap_or(terminal_size.0 / 4);
        let height = cli.height.unwrap_or(terminal_size.1 / 2);
        let settings = Settings {
            delay: cli.delay.into(), 
            hue_shift: rand::thread_rng().gen_range(0.0..360.0), 
            ansi: cli.ansi, 
        };
        let maze = Maze::new(width, height);

        // generate maze
        let mut state = generate::state(maze, settings);
        cli.generator.run(&mut state)?;

        // solve maze
        let mut state = fade::flash_between(state, solve::state)?;
        cli.solver.run(&mut state)?;
        
        // walk maze backward
        let mut state = walk::state(state);
        fade::out(&mut state)?;
        Walker::run(&mut state)?;
        
        // delay and exit
        fade::out(&mut state)
    }

    match inner() {
        Ok(_) => (),
        Err(Error::Break) => (),
        Err(Error::Io(e)) => eprintln!("{e}"), 
    }
    reset();
}
