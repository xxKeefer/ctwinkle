use crossterm::cursor::{Hide, Show};
use crossterm::style::ResetColor;
use crossterm::terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{execute, queue};
use std::io::{self, Write};

// Owns the terminal while the screensaver runs. Drop puts it back, also on panic.
pub struct Guard<W: Write> {
    out: W,
}

impl<W: Write> Guard<W> {
    // Raw mode is not part of this: the caller turns it on after `new`, and drop turns it off.
    pub fn new(out: W) -> io::Result<Self> {
        let mut g = Guard { out };
        execute!(g.out, EnterAlternateScreen, Hide, Clear(ClearType::All))?;
        Ok(g)
    }

    pub fn out(&mut self) -> &mut W {
        &mut self.out
    }
}

impl<W: Write> Drop for Guard<W> {
    fn drop(&mut self) {
        // No-op when raw mode was never turned on.
        let _ = terminal::disable_raw_mode();
        let _ = queue!(self.out, ResetColor, Show, LeaveAlternateScreen);
        let _ = self.out.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    const SHOW_CURSOR: &str = "\x1b[?25h";
    const LEAVE_ALT: &str = "\x1b[?1049l";
    const RESET_COLOUR: &str = "\x1b[0m";

    fn assert_restored(buf: &[u8]) {
        let s = String::from_utf8_lossy(buf);
        for seq in [SHOW_CURSOR, LEAVE_ALT, RESET_COLOUR] {
            assert!(s.contains(seq), "missing {seq:?} in {s:?}");
        }
    }

    #[test]
    fn new_enters_alt_screen_and_hides_cursor() {
        let mut buf = Vec::new();
        let g = Guard::new(&mut buf).unwrap();
        std::mem::forget(g);
        let s = String::from_utf8_lossy(&buf);
        assert!(s.contains("\x1b[?1049h"), "{s:?}");
        assert!(s.contains("\x1b[?25l"), "{s:?}");
    }

    #[test]
    fn drop_restores_terminal() {
        let mut buf = Vec::new();
        {
            let _g = Guard::new(&mut buf).unwrap();
        }
        assert_restored(&buf);
    }

    #[test]
    fn panic_restores_terminal() {
        let mut buf = Vec::new();
        let r = catch_unwind(AssertUnwindSafe(|| {
            let _g = Guard::new(&mut buf).unwrap();
            panic!("boom");
        }));
        assert!(r.is_err());
        assert_restored(&buf);
    }
}
