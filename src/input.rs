use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
}

pub fn map_key(key: KeyEvent) -> Option<Action> {
    match (key.code, key.modifiers) {
        (KeyCode::Char('q'), KeyModifiers::NONE) | (KeyCode::Esc, _) => Some(Action::Quit),
        (KeyCode::Char('c'), m) if m.contains(KeyModifiers::CONTROL) => Some(Action::Quit),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode, mods: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, mods)
    }

    #[test]
    fn quit_keys() {
        let quit = Some(Action::Quit);
        assert_eq!(map_key(key(KeyCode::Char('q'), KeyModifiers::NONE)), quit);
        assert_eq!(map_key(key(KeyCode::Esc, KeyModifiers::NONE)), quit);
        assert_eq!(
            map_key(key(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            quit
        );
    }

    #[test]
    fn other_keys_do_not_quit() {
        assert_eq!(map_key(key(KeyCode::Char('Q'), KeyModifiers::SHIFT)), None);
        assert_eq!(map_key(key(KeyCode::Char('x'), KeyModifiers::NONE)), None);
        assert_eq!(map_key(key(KeyCode::Char('c'), KeyModifiers::NONE)), None);
    }
}
