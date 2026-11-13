use crate::palette::Palette;
use clap::Parser;
use std::time::Duration;

#[derive(Debug, Parser)]
#[command(name = "ctwinkle", version, about = "Terminal starfield screensaver")]
pub struct Args {
    /// Star colour
    #[arg(short = 'C', long, value_enum, default_value_t = Palette::White)]
    pub colour: Palette,

    /// Mix red, green, and white stars (same as -C christmas)
    #[arg(long, conflicts_with = "colour")]
    pub christmas: bool,

    /// Rain mode: stars fall down the screen
    #[arg(short = 'r', long)]
    pub rain: bool,

    /// Frame delay, 0 (fastest, 20 ms) to 9 (slowest, 290 ms)
    #[arg(short = 'u', long, default_value_t = 4, value_parser = clap::value_parser!(u8).range(0..=9))]
    pub speed: u8,

    /// Percent of cells that hold a star, 1 to 100
    #[arg(short = 'd', long, default_value_t = 5, value_parser = clap::value_parser!(u8).range(1..=100))]
    pub density: u8,
}

impl Args {
    pub fn delay(&self) -> Duration {
        Duration::from_millis(20 + 30 * u64::from(self.speed))
    }

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
    fn rain_flag() {
        assert!(!parse(&[]).unwrap().rain);
        assert!(parse(&["-r"]).unwrap().rain);
        assert!(parse(&["--rain"]).unwrap().rain);
    }

    #[test]
    fn speed_bounds() {
        assert!(parse(&["-u", "-1"]).is_err());
        assert_eq!(parse(&["-u", "0"]).unwrap().speed, 0);
        assert_eq!(parse(&["--speed", "9"]).unwrap().speed, 9);
        assert!(parse(&["-u", "10"]).is_err());
    }

    #[test]
    fn speed_defaults_to_4() {
        assert_eq!(parse(&[]).unwrap().speed, 4);
    }

    #[test]
    fn delay_is_20ms_plus_30ms_per_step() {
        let delay = |u: &str| parse(&["-u", u]).unwrap().delay();
        assert_eq!(delay("0"), Duration::from_millis(20));
        assert_eq!(delay("4"), Duration::from_millis(140));
        assert_eq!(delay("9"), Duration::from_millis(290));
    }

    #[test]
    fn density_bounds() {
        assert!(parse(&["-d", "0"]).is_err());
        assert_eq!(parse(&["-d", "1"]).unwrap().density, 1);
        assert_eq!(parse(&["--density", "100"]).unwrap().density, 100);
        assert!(parse(&["-d", "101"]).is_err());
    }

    #[test]
    fn density_defaults_to_5() {
        assert_eq!(parse(&[]).unwrap().density, 5);
    }

    #[test]
    fn unknown_colour_is_rejected() {
        assert!(parse(&["-C", "purple"]).is_err());
    }
}
