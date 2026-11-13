pub const GLYPHS: [char; 4] = ['.', '+', '*', '✦'];

#[cfg(test)]
mod tests {
    use super::*;
    use unicode_width::UnicodeWidthChar;

    #[test]
    fn every_glyph_is_one_column_in_any_terminal() {
        for g in GLYPHS {
            assert_eq!(g.width(), Some(1), "{g:?}");
            assert_eq!(g.width_cjk(), Some(1), "{g:?} is ambiguous width");
        }
    }
}
