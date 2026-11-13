use crate::field::{Field, Level};
use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::Print;
use std::io::{self, Write};

pub const GLYPHS: [char; 4] = ['.', '+', '*', '✦'];

// One plain string per row, each exactly the field width.
pub fn frame(field: &Field) -> Vec<String> {
    let (w, h) = field.size();
    let mut grid = vec![vec![' '; w as usize]; h as usize];
    for s in field.stars() {
        grid[s.y as usize][s.x as usize] = match s.level() {
            Level::Dark => GLYPHS[0],
            Level::Dim => GLYPHS[1],
            // Mix the two bright glyphs across the field.
            Level::Bright => GLYPHS[2 + ((s.x ^ s.y) & 1) as usize],
        };
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
    fn every_glyph_is_one_column_in_any_terminal() {
        for g in GLYPHS {
            assert_eq!(g.width(), Some(1), "{g:?}");
            assert_eq!(g.width_cjk(), Some(1), "{g:?} is ambiguous width");
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
                    assert_eq!(r.width_cjk(), w as usize, "{r:?}");
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
            assert!(GLYPHS.contains(&c), "star at {},{} shows {c:?}", s.x, s.y);
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
