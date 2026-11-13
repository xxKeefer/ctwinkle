use clap::ValueEnum;
use crossterm::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Palette {
    Green,
    Red,
    Blue,
    White,
    Yellow,
    Cyan,
    Magenta,
    Black,
    Christmas,
}

const CHRISTMAS: [Color; 3] = [Color::Red, Color::Green, Color::White];

impl Palette {
    // The `c` key steps through the palettes in declared order, and wraps.
    pub fn next(self) -> Self {
        let all = Self::value_variants();
        all[(self as usize + 1) % all.len()]
    }

    // Christmas picks by the star key, so a falling star keeps its colour.
    pub fn color(self, key: u32) -> Color {
        match self {
            Palette::Green => Color::Green,
            Palette::Red => Color::Red,
            Palette::Blue => Color::Blue,
            Palette::White => Color::White,
            Palette::Yellow => Color::Yellow,
            Palette::Cyan => Color::Cyan,
            Palette::Magenta => Color::Magenta,
            Palette::Black => Color::Black,
            Palette::Christmas => {
                let mut h = u64::from(key).wrapping_mul(0xD6E8_FEB8_6659_FD93);
                h ^= h >> 32;
                CHRISTMAS[(h % CHRISTMAS.len() as u64) as usize]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn each_named_colour_parses() {
        let names = [
            ("green", Palette::Green, Color::Green),
            ("red", Palette::Red, Color::Red),
            ("blue", Palette::Blue, Color::Blue),
            ("white", Palette::White, Color::White),
            ("yellow", Palette::Yellow, Color::Yellow),
            ("cyan", Palette::Cyan, Color::Cyan),
            ("magenta", Palette::Magenta, Color::Magenta),
            ("black", Palette::Black, Color::Black),
        ];
        for (name, palette, color) in names {
            assert_eq!(Palette::from_str(name, false), Ok(palette), "{name}");
            assert_eq!(palette.color(37), color, "{name}");
        }
        assert_eq!(
            Palette::from_str("christmas", false),
            Ok(Palette::Christmas)
        );
    }

    #[test]
    fn unknown_names_are_rejected() {
        for name in ["purple", ""] {
            assert!(Palette::from_str(name, false).is_err(), "{name:?}");
        }
    }

    #[test]
    fn next_steps_in_order_and_wraps() {
        assert_eq!(Palette::Green.next(), Palette::Red);
        assert_eq!(Palette::Black.next(), Palette::Christmas);
        assert_eq!(Palette::Christmas.next(), Palette::Green);
    }

    #[test]
    fn christmas_mixes_only_red_green_white() {
        let mut seen = HashSet::new();
        for key in 0..1000 {
            let c = Palette::Christmas.color(key);
            assert!(CHRISTMAS.contains(&c), "{c:?}");
            seen.insert(format!("{c:?}"));
        }
        assert_eq!(seen.len(), 3, "{seen:?}");
    }
}
