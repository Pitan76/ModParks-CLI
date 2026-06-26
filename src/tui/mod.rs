// src/tui/mod.rs
pub mod app;
pub mod events;
pub mod handlers;
pub mod ui;
pub mod screens;

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

use crate::config::Config;
use self::app::{App, Screen, InputMode, LoginType};
use self::ui::{COLOR_BG, split_layout};
use self::events::{poll_event, AppEvent};
use self::handlers::{handle_key_event, fetch_projects};

pub async fn run_tui() -> Result<()> {
    let cfg = Config::load()?;
    
    let current_user = if !cfg.api_key.is_empty() {
        crate::api_client::auth_me(&cfg).await.ok()
    } else {
        None
    };
    
    let mut app = App::new(cfg);
    app.current_user = current_user;
    
    if app.screen == Screen::ProjectList {
        fetch_projects(&mut app).await;
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| {
            let area = f.area();
            f.render_widget(ratatui::widgets::Block::default().style(ratatui::style::Style::default().bg(COLOR_BG)), area);
            let (header_area, body_area, footer_area) = split_layout(area);

            let screen_clone = app.screen.clone();
            match screen_clone {
                Screen::Login => screens::login::render(&app, f, header_area, body_area, footer_area),
                Screen::ProjectList => screens::project_list::render(&mut app, f, header_area, body_area, footer_area),
                Screen::MyProjects => screens::my_projects::render(&mut app, f, header_area, body_area, footer_area),
                Screen::Profile => screens::profile::render(&app, f, header_area, body_area, footer_area, None),
                Screen::ProjectDetail(idx) | Screen::MyProjectDetail(idx) => {
                    screens::project_detail::render(&app, f, header_area, body_area, footer_area, idx);
                }
                Screen::ProjectCreate | Screen::ProjectEdit(_) => {
                    screens::project_form::render(&app, f, header_area, body_area, footer_area);
                }
                Screen::IdeaList => screens::idea_list::render(&mut app, f, header_area, body_area, footer_area),
                Screen::IdeaDetail(idx) => screens::idea_detail::render(&app, f, header_area, body_area, footer_area, idx),
                Screen::Help => screens::help::render(&app, f, header_area, body_area, footer_area),
            }
        })?;

                if let Some(app_event) = poll_event()? {
            match app_event {
                AppEvent::Key(key) => {
                    if handle_key_event(&mut app, key).await { break; }
                }
                AppEvent::Tick => {}
            }
        }
    }
disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}