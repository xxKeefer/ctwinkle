use crate::cli;
use crate::field::{Field, Mode};
use crate::input::{map_key, Action};
use crate::palette::Palette;
use crate::render;
use crossterm::event::Event;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

pub struct App {
    pub field: Field,
    pub palette: Palette,
    pub frame: Duration,
}

impl App {
    // Gives false when the action ends the run.
    pub fn apply(&mut self, action: Action) -> bool {
        match action {
            Action::Quit => return false,
            Action::Speed(n) => self.frame = cli::delay(n),
            Action::Rain => self.field.set_mode(match self.field.mode() {
                Mode::Twinkle => Mode::Rain,
                Mode::Rain => Mode::Twinkle,
            }),
            Action::Colour => self.palette = self.palette.next(),
        }
        true
    }
}

// `poll` waits up to the given time for one event, and gives None on timeout.
pub fn run<W: Write>(
    out: &mut W,
    app: &mut App,
    stop: &AtomicBool,
    mut poll: impl FnMut(Duration) -> io::Result<Option<Event>>,
) -> io::Result<()> {
    while !stop.load(Ordering::Relaxed) {
        render::draw(out, &app.field, app.palette)?;
        let end = Instant::now() + app.frame;
        while let Some(event) = poll(end.saturating_duration_since(Instant::now()))? {
            match event {
                Event::Key(key) => {
                    if let Some(action) = map_key(key) {
                        if !app.apply(action) {
                            return Ok(());
                        }
                    }
                }
                Event::Resize(w, h) => app.field.resize(w, h),
                _ => {}
            }
            if stop.load(Ordering::Relaxed) {
                return Ok(());
            }
        }
        app.field.tick();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    const LIMIT: usize = 100;
    const FRAME: Duration = Duration::from_millis(80);

    fn app(frame: Duration) -> App {
        App {
            field: Field::new(10, 5, 10, 1),
            palette: Palette::White,
            frame,
        }
    }

    fn key(c: char) -> Event {
        Event::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE))
    }

    #[test]
    fn stop_flag_ends_loop_on_next_tick() {
        let stop = AtomicBool::new(false);
        let mut app = app(FRAME);
        let mut out = Vec::new();
        let mut calls = 0;
        run(&mut out, &mut app, &stop, |_| {
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
        let mut app = app(FRAME);
        let mut calls = 0;
        run(&mut Vec::new(), &mut app, &stop, |_| {
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
        let mut app = app(FRAME);
        let mut calls = 0;
        run(&mut Vec::new(), &mut app, &stop, |_| {
            calls += 1;
            assert!(calls < LIMIT, "loop did not stop");
            if calls == 5 {
                stop.store(true, Ordering::Relaxed);
                return Ok(None);
            }
            Ok(Some(key('x')))
        })
        .unwrap();
        assert_eq!(calls, 5);
    }

    #[test]
    fn resize_event_resizes_field() {
        let stop = AtomicBool::new(false);
        let mut app = app(FRAME);
        let mut calls = 0;
        run(&mut Vec::new(), &mut app, &stop, |_| {
            calls += 1;
            assert!(calls < LIMIT, "loop did not stop");
            if calls == 1 {
                return Ok(Some(Event::Resize(30, 20)));
            }
            stop.store(true, Ordering::Relaxed);
            Ok(None)
        })
        .unwrap();
        assert_eq!(app.field.size(), (30, 20));
    }

    #[test]
    fn poll_waits_up_to_the_given_frame() {
        let stop = AtomicBool::new(false);
        let frame = Duration::from_millis(500);
        let mut app = app(frame);
        let mut waits = Vec::new();
        run(&mut Vec::new(), &mut app, &stop, |t| {
            waits.push(t);
            assert!(waits.len() < LIMIT, "loop did not stop");
            stop.store(true, Ordering::Relaxed);
            Ok(None)
        })
        .unwrap();
        assert!(waits[0] <= frame && waits[0] > frame / 2, "{:?}", waits[0]);
    }

    #[test]
    fn quit_action_ends_run() {
        assert!(!app(FRAME).apply(Action::Quit));
    }

    #[test]
    fn speed_action_sets_frame_delay() {
        let mut app = app(FRAME);
        for n in 0..=9 {
            assert!(app.apply(Action::Speed(n)));
            assert_eq!(app.frame, cli::delay(n), "speed {n}");
        }
        assert_eq!(app.frame, Duration::from_millis(290));
    }

    #[test]
    fn rain_action_toggles_and_returns() {
        let mut app = app(FRAME);
        assert!(app.apply(Action::Rain));
        assert_eq!(app.field.mode(), Mode::Rain);
        assert!(app.apply(Action::Rain));
        assert_eq!(app.field.mode(), Mode::Twinkle);
    }

    #[test]
    fn colour_action_wraps_after_9_steps() {
        let mut app = app(FRAME);
        let mut seen = vec![app.palette];
        for _ in 0..9 {
            assert!(app.apply(Action::Colour));
            seen.push(app.palette);
        }
        assert_eq!(app.palette, Palette::White);
        seen.pop();
        seen.sort_by_key(|p| *p as u8);
        seen.dedup();
        assert_eq!(seen.len(), 9, "{seen:?}");
    }

    #[test]
    fn keys_change_the_running_app_on_the_next_frame() {
        let stop = AtomicBool::new(false);
        let mut app = app(FRAME);
        let mut waits = Vec::new();
        let events = [key('0'), key('r'), key('c'), key('c')];
        run(&mut Vec::new(), &mut app, &stop, |t| {
            waits.push(t);
            assert!(waits.len() < LIMIT, "loop did not stop");
            match events.get(waits.len() - 1) {
                Some(e) => Ok(Some(e.clone())),
                None if waits.len() == events.len() + 1 => Ok(None),
                None => {
                    stop.store(true, Ordering::Relaxed);
                    Ok(None)
                }
            }
        })
        .unwrap();
        assert_eq!(app.frame, Duration::from_millis(20));
        assert_eq!(app.field.mode(), Mode::Rain);
        assert_eq!(app.palette, Palette::Cyan);
        let next = waits[events.len() + 1];
        assert!(next <= Duration::from_millis(20), "{next:?}");
    }
}
