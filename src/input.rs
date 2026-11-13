use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    Speed(u8),
    Rain,
    Colour,
}

pub fn map_key(key: KeyEvent) -> Option<Action> {
    match (key.code, key.modifiers) {
        (KeyCode::Char('q'), KeyModifiers::NONE) | (KeyCode::Esc, _) => Some(Action::Quit),
        (KeyCode::Char('c'), m) if m.contains(KeyModifiers::CONTROL) => Some(Action::Quit),
        (KeyCode::Char(c @ '0'..='9'), KeyModifiers::NONE) => Some(Action::Speed(c as u8 - b'0')),
        (KeyCode::Char('r'), KeyModifiers::NONE) => Some(Action::Rain),
        (KeyCode::Char('c'), KeyModifiers::NONE) => Some(Action::Colour),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode, mods: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, mods)
    }

    fn plain(c: char) -> Option<Action> {
        map_key(key(KeyCode::Char(c), KeyModifiers::NONE))
    }

    #[test]
    fn quit_keys() {
        let quit = Some(Action::Quit);
        assert_eq!(plain('q'), quit);
        assert_eq!(map_key(key(KeyCode::Esc, KeyModifiers::NONE)), quit);
        assert_eq!(
            map_key(key(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            quit
        );
    }

    #[test]
    fn other_keys_do_not_quit() {
        assert_eq!(map_key(key(KeyCode::Char('Q'), KeyModifiers::SHIFT)), None);
        assert_eq!(plain('x'), None);
        assert_ne!(plain('c'), Some(Action::Quit));
    }

    #[test]
    fn digit_keys_set_speed() {
        for (c, n) in ('0'..='9').zip(0..) {
            assert_eq!(plain(c), Some(Action::Speed(n)), "{c}");
        }
    }

    #[test]
    fn r_and_c_keys() {
        assert_eq!(plain('r'), Some(Action::Rain));
        assert_eq!(plain('c'), Some(Action::Colour));
        assert_eq!(
            map_key(key(KeyCode::Char('r'), KeyModifiers::CONTROL)),
            None
        );
    }
}
