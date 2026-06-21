// src/tui/events.rs
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::time::Duration;

pub enum AppEvent {
    Key(KeyEvent),
    Tick,
}

pub fn poll_event() -> Result<Option<AppEvent>> {
    if event::poll(Duration::from_millis(50))? {
        let ev = event::read()?;
        if let Event::Key(key) = ev {
            if key.kind == KeyEventKind::Press {
                return Ok(Some(AppEvent::Key(key)));
            }
        }
    }
    Ok(Some(AppEvent::Tick))
}

pub fn is_quit(key: &KeyEvent) -> bool {
    key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL)
}