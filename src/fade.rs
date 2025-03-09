use std::marker::PhantomData;
use crate::{
    colour::Palette, 
    state::{Phase, State}, 
    Animation, Error, Signal, 
};

pub fn flash_between<T: Phase, U: Phase>(
    mut prev_state: State<T>,
    next: impl FnOnce(State<T>) -> State<U>, 
) -> Result<State<U>, Error> {
    Fade::<T, 5>::run(&mut prev_state)?;

    let prev_palette = prev_state.colours.palette;
    let mut state = next(prev_state);
    let next_palette = state.colours.palette;

    if state.settings.ansi {
        Fade::<U>::run(&mut state)?;
        return Ok(state)
    }
    
    let flash_colours = {
        let young = prev_palette.young;
        let old = next_palette.unvisited.unwrap();
        Palette::new(young, old)
            .with_maybe_special(next_palette.special)
            .into_lut(&state.settings)
    };
    state.age.fill(Some(0));
    
    // run fade in the next state (to get special nodes coloured) but with our custom colours
    let colours = state.colours.clone();
    state.colours = flash_colours;
    Fade::<U>::run(&mut state)?;

    // restore and return state
    state.age.fill(None);
    state.colours = colours;
    Ok(state)
}

pub fn out<T: Phase>(state: &mut State<T>) -> Result<(), Error> {
    Fade::<T>::run(state)
}

struct Fade<T, const STEPS: u8 = 255> {
    steps: u8, 
    _phase: PhantomData<T>,
}

impl<T: Phase, const STEPS: u8> Animation for Fade<T, STEPS> {
    type Phase = T;

    fn new(_state: &mut State<T>) -> Self {
        Fade {
            steps: STEPS, 
            _phase: PhantomData, 
        }
    }

    fn step(&mut self, _state: &mut State<T>) -> Signal {
        match self.steps.checked_sub(1) {
            Some(steps) => {
                self.steps = steps;
                Signal::Continue
            }, 
            None => Signal::Done, 
        }
    }

    fn timescale(&self) -> u32 {
        350
    }
}
