use crate::field::{pick, Field, Level, Star};
use crate::palette::Palette;
use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::{Print, SetForegroundColor};
use std::io::{self, Write};

// Glyph groups from dim to bright, one per `Level`.
pub const GLYPHS: [&[char]; 3] = [
    &['.', '·', '°'],
    &['+', '•', '✧', '☆'],
    &['*', '✦', '✶', '✷', '✸', '✵', '✯', '★'],
];

// Keyed on the star, not its cell, so a falling star keeps its glyph.
pub fn glyph(key: u32, level: Level) -> char {
    pick(key, 0x9E37_79B9_7F4A_7C15, GLYPHS[level as usize])
}

fn grid(field: &Field) -> Vec<Vec<Option<&Star>>> {
    let (w, h) = field.size();
    let mut grid = vec![vec![None; w as usize]; h as usize];
    for s in field.stars() {
        grid[s.y as usize][s.x as usize] = Some(s);
    }
    grid
}

pub fn draw<W: Write>(out: &mut W, field: &Field, palette: Palette) -> io::Result<()> {
    for (y, row) in grid(field).into_iter().enumerate() {
        queue!(out, MoveTo(0, y as u16))?;
        for star in row {
            match star {
                Some(s) => queue!(
                    out,
                    SetForegroundColor(palette.color(s.key())),
                    Print(glyph(s.key(), s.level()))
                )?,
                None => queue!(out, Print(' '))?,
            }
        }
    }
    out.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Mode;
    use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

    // One plain string per row, each exactly the field width.
    fn frame(field: &Field) -> Vec<String> {
        grid(field)
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|s| s.map_or(' ', |s| glyph(s.key(), s.level())))
                    .collect()
            })
            .collect()
    }

    #[test]
    fn every_glyph_is_one_column() {
        for &g in GLYPHS.iter().copied().flatten() {
            assert_eq!(g.width(), Some(1), "{g:?}");
        }
    }

    fn group(level: Level) -> &'static [char] {
        match level {
            Level::Dark => &['.', '·', '°'],
            Level::Dim => &['+', '•', '✧', '☆'],
            Level::Bright => &['*', '✦', '✶', '✷', '✸', '✵', '✯', '★'],
        }
    }

    #[test]
    fn glyph_comes_from_its_level_group_and_uses_the_whole_group() {
        for level in [Level::Dark, Level::Dim, Level::Bright] {
            let mut seen = std::collections::HashSet::new();
            for key in 0..1600 {
                let g = glyph(key, level);
                assert!(group(level).contains(&g), "{level:?} gave {g:?}");
                assert_eq!(g, glyph(key, level), "glyph not stable");
                seen.insert(g);
            }
            assert_eq!(seen.len(), group(level).len(), "{level:?} used {seen:?}");
        }
    }

    #[test]
    fn every_row_is_field_width() {
        for (w, h, d) in [(80, 24, 5), (13, 7, 100), (1, 1, 100), (0, 0, 5)] {
            let mut f = Field::new(w, h, d, 2);
            for _ in 0..20 {
                let rows = frame(&f);
                assert_eq!(rows.len(), h as usize);
                for r in &rows {
                    assert_eq!(r.width(), w as usize, "{r:?}");
                }
                let mut buf = Vec::new();
                draw(&mut buf, &f, Palette::White).unwrap();
                let drawn = cells(&buf);
                assert_eq!(drawn.len(), h as usize);
                for r in &drawn {
                    let s: String = r.iter().map(|c| c.1).collect();
                    assert_eq!(s.width(), w as usize, "{s:?}");
                }
                f.tick();
            }
        }
    }

    #[test]
    fn frame_shows_each_star() {
        let f = Field::new(20, 10, 30, 4);
        let rows: Vec<Vec<char>> = frame(&f).iter().map(|r| r.chars().collect()).collect();
        for s in f.stars() {
            let c = rows[s.y as usize][s.x as usize];
            assert_eq!(c, glyph(s.key(), s.level()), "star at {},{}", s.x, s.y);
        }
        let lit = rows.iter().flatten().filter(|&&c| c != ' ').count();
        assert_eq!(lit, f.stars().len());
    }

    fn fg(c: crossterm::style::Color) -> String {
        let mut b = Vec::new();
        queue!(b, crossterm::style::SetForegroundColor(c)).unwrap();
        String::from_utf8(b).unwrap()
    }

    // The escape bytes for one row, built from the stars in the field.
    fn expected_row(f: &Field, y: u16, palette: Palette) -> String {
        let (w, _) = f.size();
        let mut row = format!("\x1b[{};1H", y + 1);
        for x in 0..w {
            match f.stars().iter().find(|s| (s.x, s.y) == (x, y)) {
                Some(s) => {
                    row.push_str(&fg(palette.color(s.key())));
                    row.push(glyph(s.key(), s.level()));
                }
                None => row.push(' '),
            }
        }
        row
    }

    #[test]
    fn draw_positions_and_colours_each_row() {
        for palette in [Palette::White, Palette::Christmas] {
            let f = Field::new(12, 6, 40, 3);
            let mut buf = Vec::new();
            draw(&mut buf, &f, palette).unwrap();
            let s = String::from_utf8_lossy(&buf);
            for y in 0..6 {
                assert!(s.contains(&expected_row(&f, y, palette)), "row {y}: {s:?}");
            }
        }
    }

    #[test]
    fn draw_writes_the_literal_colour_and_a_glyph_of_the_star_level() {
        let f = Field::new(1, 1, 100, 5);
        let mut buf = Vec::new();
        draw(&mut buf, &f, Palette::Green).unwrap();
        let s = String::from_utf8(buf).unwrap();
        let g = s.strip_prefix("\x1b[1;1H\x1b[38;5;10m").expect(&s);
        let mut chars = g.chars();
        let c = chars.next().unwrap();
        assert_eq!(chars.next(), None, "{s:?}");
        assert!(group(f.stars()[0].level()).contains(&c), "{c:?}");
    }

    // Reads `draw` output back into (colour escape, glyph) per cell.
    fn cells(buf: &[u8]) -> Vec<Vec<(String, char)>> {
        let s = String::from_utf8_lossy(buf);
        let mut rows: Vec<Vec<(String, char)>> = Vec::new();
        let mut colour = String::new();
        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            if c != '\x1b' {
                rows.last_mut().unwrap().push((colour.clone(), c));
                continue;
            }
            let mut esc = String::from(c);
            for e in chars.by_ref() {
                esc.push(e);
                if e.is_ascii_alphabetic() {
                    break;
                }
            }
            match esc.chars().last() {
                Some('H') => rows.push(Vec::new()),
                Some('m') => colour = esc,
                _ => {}
            }
        }
        rows
    }

    #[test]
    fn falling_star_keeps_its_glyph_and_colour() {
        let mut f = Field::new(10, 40, 10, 8);
        f.set_mode(Mode::Rain);
        let drawn = |f: &Field| {
            let mut buf = Vec::new();
            draw(&mut buf, f, Palette::Christmas).unwrap();
            cells(&buf)
        };
        let mut checked = 0;
        for _ in 0..30 {
            let old = drawn(&f);
            let before: Vec<_> = f.stars().iter().map(|s| (s.x, s.y, s.level())).collect();
            f.tick();
            let new = drawn(&f);
            for (s, &(x, y, level)) in f.stars().iter().zip(&before) {
                if (s.x, s.y) != (x, y + 1) {
                    continue;
                }
                let (was, now) = (
                    &old[y as usize][x as usize],
                    &new[s.y as usize][s.x as usize],
                );
                assert_eq!(was.0, now.0, "colour changed while falling");
                if s.level() == level {
                    assert_eq!(was.1, now.1, "glyph changed while falling");
                    checked += 1;
                }
            }
        }
        assert!(checked > 100, "{checked}");
    }
}
