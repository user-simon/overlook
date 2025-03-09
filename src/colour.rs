use crossterm::style::Color as CrosstermColour;
use palette::{FromColor, Okhsl, OklabHue, Srgb};
use crate::Settings;

/// Represents a colour with HSL coordinates. 
///
/// The colours are rendered to the terminal via [`Colour::to_crossterm`] in the Okhsl colour space. 
#[derive(Clone, Copy, Debug, Default)]
pub struct Hsl {
    pub hue: f64,
    pub saturation: f64,
    pub lightness: f64, 
}

impl Hsl {
    pub const fn with_h(self, hue: f64) -> Hsl {
        let hue = wrap(hue, 360.0);
        Hsl{ hue, ..self }
    }

    pub const fn with_s(self, saturation: f64) -> Hsl {
        let saturation = clamp(saturation, 0.0, 1.0);
        Hsl{ saturation, ..self }
    }

    pub const fn with_l(self, lightness: f64) -> Hsl {
        let lightness = clamp(lightness, 0.0, 1.0);
        Hsl{ lightness, ..self }
    }

    pub const fn shift_h(self, hue: f64) -> Hsl {
        self.with_h(self.hue + hue)
    }

    pub const fn shift_s(self, saturation: f64) -> Hsl {
        self.with_s(self.saturation + saturation)
    }

    pub const fn shift_l(self, lightness: f64) -> Hsl {
        self.with_l(self.lightness + lightness)
    }

    fn to_crossterm(self) -> CrosstermColour {
        // convert to palette::Okhsl (we're not using this type directly to provide a better interface)
        let Hsl{ hue, lightness, saturation } = self;
        let okhsl = Okhsl {
            hue: OklabHue::new(hue),
            saturation,
            lightness, 
        };
        // convert Okhsl to SRGB
        let (r, g, b) = Srgb::from_color(okhsl).into_components();

        // convert SRGB to crossterm::Color
        let [r, g, b] = [r, g, b].map(|x| (x * 255.0) as u8);
        CrosstermColour::Rgb{ r, g, b }
    }
}

/// Colour lookup derived from [`Palette`] to be used when rendering maze nodes. 
#[derive(Clone)]
pub struct Lut {
    /// Palette the LUT is derived from. 
    pub palette: Palette, 
    /// Precomputed age gradient. 
    pub gradient: [CrosstermColour; 256], 
    /// Colour of unvisited nodes. 
    pub unvisited: CrosstermColour, 
    /// Colour of special nodes. 
    pub special: CrosstermColour, 
}

impl Lut {
    /// Gets the colour of a node by sampling the gradient if the node is unvisited. 
    pub fn sample(&self, age: Option<u8>) -> CrosstermColour {
        age.map(|i| self.gradient[i as usize]).unwrap_or(self.unvisited)
    }
}

/// Colour palette used to derive a [`Lut`]. 
#[derive(Clone, Copy, Debug)]
pub struct Palette {
    /// Colour that the age gradient is "based" on. 
    pub base: Hsl, 
    /// Colour of the youngest nodes. 
    pub young: Hsl, 
    /// Colour of the oldest nodes. 
    pub old: Hsl, 
    /// Colour unvisited nodes. 
    pub unvisited: Option<Hsl>, 
    /// Colour of special nodes (as defined by [`Phase::special`](crate::state::Phase::special)). 
    pub special: Option<Hsl>, 
}

impl Palette {
    /// Derives an age gradient from a "base" colour. 
    pub fn from_base(base: Hsl) -> Palette {
        let old = base
            .with_l(0.3);
        let young = base
            .shift_h(60.0);
        Palette {
            base, 
            young, 
            old, 
            unvisited: None, 
            special: None, 
        }
    }

    /// Constructs a new palette with given age gradient poles. 
    pub fn new(young: Hsl, old: Hsl) -> Palette {
        Palette {
            base: old.with_l(0.75).with_s(0.5), 
            young, 
            old, 
            unvisited: None, 
            special: None, 
        }
    }

    pub fn with_unvisited(self, unvisited: Hsl) -> Palette {
        let unvisited = Some(unvisited);
        Palette{ unvisited, ..self }
    }

    pub fn with_maybe_special(self, special: Option<Hsl>) -> Palette {
        Palette{ special, ..self }
    }

    pub fn with_special(self, special: Hsl) -> Palette {
        let special = Some(special);
        Palette{ special, ..self }
    }

    /// Derives a [`Lut`] from the palette. Note that the palette may be overriden by settings such as
    /// [`Settings::ansi`]. 
    pub fn into_lut(self, settings: &Settings) -> Lut {
        if settings.ansi {
            return self.ansi()
        }
        
        let normalise = |colour: Hsl| colour
            .shift_h(settings.hue_shift)
            .to_crossterm();
        let ease = |t| (1.0 - f64::powi(t - 1.0, 2)).powf(1.0/3.0); // slightly more aggressive outCirc
        let gradient = std::array::from_fn(|i| {
            let t = ease(i as f64 / 255.0);
            normalise(lerp(self.young, self.old, t))
        });

        let unvisited = self.unvisited
            .map(normalise)
            .unwrap_or(CrosstermColour::Reset);
        let special = self.special
            .map(normalise)
            .unwrap_or(CrosstermColour::Reset);

        Lut {
            palette: self, 
            gradient, 
            unvisited,
            special,
        }
    }

    /// Derives a [`Lut`] using only standard ANSI colours (not RGB), ignoring most of the palette. 
    fn ansi(self) -> Lut {
        let mut gradient = [CrosstermColour::White; 256];
        let unvisited = self.unvisited
            .map(|_| CrosstermColour::DarkGrey)
            .unwrap_or(CrosstermColour::Reset);
        let special = self.special
            .map(|_| CrosstermColour::Blue)
            .unwrap_or(CrosstermColour::Reset);

        for i in 0..4 {
            gradient[i] = CrosstermColour::DarkRed;
        }
        for i in 4..8 {
            gradient[i] = CrosstermColour::Red;
        }

        Lut {
            palette: self,
            gradient,
            unvisited,
            special, 
        }
    }
}

/// Linearly interpolates between two colours using a time value between 0 and 1. 
fn lerp(a: Hsl, b: Hsl, t: f64) -> Hsl {
    let lerp_component = |x, y| x + t * (y - x);

    Hsl {
        hue: lerp_component(a.hue, b.hue), 
        saturation: lerp_component(a.saturation, b.saturation), 
        lightness: lerp_component(a.lightness, b.lightness), 
    }
}

/// The world's least intelligent euclidian remainder implementation (we can't use [`f64::rem_euclid`] since
/// it's not const). That being said, the performance is good if `x` is within one or two multiples of `max`,
/// which we can reasonably expect. 
const fn wrap(x: f64, max: f64) -> f64 {
    if x < 0.0 {
        wrap(max - x, max)
    } else if x > max {
        wrap(x - max, max)
    } else {
        x
    }
}

/// Clamps a value to a range. 
const fn clamp(x: f64, min: f64, max: f64) -> f64 {
    if x > max {
        max
    } else if x < min {
        min
    } else {
        x
    }
}
