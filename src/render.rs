use crate::field::{Field, Level};
use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::Print;
use std::io::{self, Write};

// Glyph groups from dim to bright, one per `Level`.
pub const GLYPHS: [&[char]; 3] = [
    &['.', '·', '°'],
    &['+', '•', '✧', '☆'],
    &['*', '✦', '✶', '✷', '✸', '✵', '✯', '★'],
];

// A star keeps its cell for one twinkle cycle, so a position hash keeps its glyph stable.
pub fn glyph(x: u16, y: u16, level: Level) -> char {
    let group = GLYPHS[level as usize];
    let mut h = (u64::from(x) << 16 | u64::from(y)).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^= h >> 32;
    group[(h % group.len() as u64) as usize]
}

// One plain string per row, each exactly the field width.
pub fn frame(field: &Field) -> Vec<String> {
    let (w, h) = field.size();
    let mut grid = vec![vec![' '; w as usize]; h as usize];
    for s in field.stars() {
        grid[s.y as usize][s.x as usize] = glyph(s.x, s.y, s.level());
    }
    grid.into_iter().map(String::from_iter).collect()
}

pub fn draw<W: Write>(out: &mut W, rows: &[String]) -> io::Result<()> {
    for (y, row) in rows.iter().enumerate() {
        queue!(out, MoveTo(0, y as u16), Print(row))?;
    }
    out.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

    #[test]
    fn every_glyph_is_one_column() {
        for &g in GLYPHS.iter().copied().flatten() {
            assert_eq!(g.width(), Some(1), "{g:?}");
        }
    }

    fn group(level: Level) -> &'static [char] {
        match level {
            Level::Dark => GLYPHS[0],
            Level::Dim => GLYPHS[1],
            Level::Bright => GLYPHS[2],
        }
    }

    #[test]
    fn glyph_comes_from_its_level_group_and_uses_the_whole_group() {
        for level in [Level::Dark, Level::Dim, Level::Bright] {
            let mut seen = std::collections::HashSet::new();
            for x in 0..40 {
                for y in 0..40 {
                    let g = glyph(x, y, level);
                    assert!(group(level).contains(&g), "{level:?} gave {g:?}");
                    assert_eq!(g, glyph(x, y, level), "glyph not stable");
                    seen.insert(g);
                }
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
            assert_eq!(c, glyph(s.x, s.y, s.level()), "star at {},{}", s.x, s.y);
        }
        let lit = rows.iter().flatten().filter(|&&c| c != ' ').count();
        assert_eq!(lit, f.stars().len());
    }

    #[test]
    fn draw_positions_each_row() {
        let rows = vec!["ab".to_string(), "cd".to_string()];
        let mut buf = Vec::new();
        draw(&mut buf, &rows).unwrap();
        let s = String::from_utf8_lossy(&buf);
        assert!(s.contains("\x1b[1;1Hab"), "{s:?}");
        assert!(s.contains("\x1b[2;1Hcd"), "{s:?}");
    }
}
