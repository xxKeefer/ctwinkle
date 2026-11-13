use crate::field::Field;
use crate::input::{map_key, Action};
use crate::render;
use crossterm::event::Event;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

pub const FRAME: Duration = Duration::from_millis(80);

// `poll` waits up to the given time for one event, and gives None on timeout.
pub fn run<W: Write>(
    out: &mut W,
    field: &mut Field,
    stop: &AtomicBool,
    mut poll: impl FnMut(Duration) -> io::Result<Option<Event>>,
) -> io::Result<()> {
    while !stop.load(Ordering::Relaxed) {
        render::draw(out, &render::frame(field))?;
        let end = Instant::now() + FRAME;
        while let Some(event) = poll(end.saturating_duration_since(Instant::now()))? {
            match event {
                Event::Key(key) if map_key(key) == Some(Action::Quit) => return Ok(()),
                Event::Resize(w, h) => field.resize(w, h),
                _ => {}
            }
            if stop.load(Ordering::Relaxed) {
                return Ok(());
            }
        }
        field.tick();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    const LIMIT: usize = 100;

    #[test]
    fn stop_flag_ends_loop_on_next_tick() {
        let stop = AtomicBool::new(false);
        let mut field = Field::new(10, 5, 10, 1);
        let mut out = Vec::new();
        let mut calls = 0;
        run(&mut out, &mut field, &stop, |_| {
            calls += 1;
            assert!(calls < LIMIT, "loop did not stop");
            if calls == 3 {
                stop.store(true, Ordering::Relaxed);
            }
            Ok(None)
        })
        .unwrap();
        assert_eq!(calls, 3);
        assert!(!out.is_empty(), "no frame drawn");
    }

    #[test]
    fn quit_key_ends_loop() {
        let stop = AtomicBool::new(false);
        let mut field = Field::new(10, 5, 10, 1);
        let mut calls = 0;
        run(&mut Vec::new(), &mut field, &stop, |_| {
            calls += 1;
            assert!(calls < LIMIT, "loop did not stop");
            Ok(Some(Event::Key(KeyEvent::new(
                KeyCode::Esc,
                KeyModifiers::NONE,
            ))))
        })
        .unwrap();
        assert_eq!(calls, 1);
    }

    #[test]
    fn other_key_keeps_running() {
        let stop = AtomicBool::new(false);
        let mut field = Field::new(10, 5, 10, 1);
        let mut calls = 0;
        run(&mut Vec::new(), &mut field, &stop, |_| {
            calls += 1;
            assert!(calls < LIMIT, "loop did not stop");
            if calls == 5 {
                stop.store(true, Ordering::Relaxed);
                return Ok(None);
            }
            Ok(Some(Event::Key(KeyEvent::new(
                KeyCode::Char('x'),
                KeyModifiers::NONE,
            ))))
        })
        .unwrap();
        assert_eq!(calls, 5);
    }

    #[test]
    fn resize_event_resizes_field() {
        let stop = AtomicBool::new(false);
        let mut field = Field::new(10, 5, 10, 1);
        let mut calls = 0;
        run(&mut Vec::new(), &mut field, &stop, |_| {
            calls += 1;
            assert!(calls < LIMIT, "loop did not stop");
            if calls == 1 {
                return Ok(Some(Event::Resize(30, 20)));
            }
            stop.store(true, Ordering::Relaxed);
            Ok(None)
        })
        .unwrap();
        assert_eq!(field.size(), (30, 20));
    }
}
