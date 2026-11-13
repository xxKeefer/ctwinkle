use crate::palette::Palette;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "ctwinkle", version, about = "Terminal starfield screensaver")]
pub struct Args {
    /// Star colour
    #[arg(short = 'C', long, value_enum, default_value_t = Palette::White)]
    pub colour: Palette,

    /// Mix red, green, and white stars (same as -C christmas)
    #[arg(long, conflicts_with = "colour")]
    pub christmas: bool,
}

impl Args {
    pub fn palette(&self) -> Palette {
        if self.christmas {
            Palette::Christmas
        } else {
            self.colour
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Args, clap::Error> {
        Args::try_parse_from([&["ctwinkle"], args].concat())
    }

    #[test]
    fn palette_defaults_to_white() {
        assert_eq!(parse(&[]).unwrap().palette(), Palette::White);
    }

    #[test]
    fn colour_flag_picks_palette() {
        assert_eq!(parse(&["-C", "green"]).unwrap().palette(), Palette::Green);
        assert_eq!(parse(&["--colour", "red"]).unwrap().palette(), Palette::Red);
    }

    #[test]
    fn christmas_by_flag_or_name() {
        assert_eq!(
            parse(&["--christmas"]).unwrap().palette(),
            Palette::Christmas
        );
        assert_eq!(
            parse(&["-C", "christmas"]).unwrap().palette(),
            Palette::Christmas
        );
    }

    #[test]
    fn unknown_colour_is_rejected() {
        assert!(parse(&["-C", "purple"]).is_err());
    }
}
