// src/tui/mod.rs
pub mod events;
pub mod ui;

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, widgets::ListState, Terminal};
use std::io;
use crossterm::event::KeyCode;

use crate::api_models::{ApiProject, ApiIdea};
use crate::api_client::cached_get;
use crate::config::Config;
use crate::pagination::PaginatedResponse;
use self::events::{poll_event, AppEvent, is_quit};
use self::ui::*;

#[derive(Debug, Clone, PartialEq)]
enum Screen {
    ProjectList,
    ProjectDetail(usize),
    IdeaList,
    IdeaDetail(usize),
    Help,
}

struct App {
    screen: Screen,
    prev_screen: Option<Screen>,
    projects: Vec<ApiProject>,
    ideas: Vec<ApiIdea>,
    project_state: ListState,
    idea_state: ListState,
    loading: bool,
    cfg: Config,
}

impl App {
    fn new(cfg: Config) -> Self {
        let mut project_state = ListState::default();
        project_state.select(Some(0));
        let mut idea_state = ListState::default();
        idea_state.select(Some(0));
        Self {
            screen: Screen::ProjectList,
            prev_screen: None,
            projects: vec![],
            ideas: vec![],
            project_state,
            idea_state,
            loading: true,
            cfg,
        }
    }

    fn go_back(&mut self) {
        if let Some(prev) = self.prev_screen.take() {
            self.screen = prev;
        } else {
            match &self.screen {
                Screen::ProjectDetail(_) => { self.screen = Screen::ProjectList; }
                Screen::IdeaDetail(_)    => { self.screen = Screen::IdeaList; }
                Screen::Help             => { self.screen = Screen::ProjectList; }
                _                        => {}
            }
        }
    }

    fn move_up(&mut self) {
        match &self.screen {
            Screen::ProjectList => {
                let i = self.project_state.selected().unwrap_or(0);
                if i > 0 { self.project_state.select(Some(i - 1)); }
            }
            Screen::IdeaList => {
                let i = self.idea_state.selected().unwrap_or(0);
                if i > 0 { self.idea_state.select(Some(i - 1)); }
            }
            _ => {}
        }
    }

    fn move_down(&mut self) {
        match &self.screen {
            Screen::ProjectList => {
                let len = self.projects.len();
                if len == 0 { return; }
                let i = self.project_state.selected().unwrap_or(0);
                if i < len - 1 { self.project_state.select(Some(i + 1)); }
            }
            Screen::IdeaList => {
                let len = self.ideas.len();
                if len == 0 { return; }
                let i = self.idea_state.selected().unwrap_or(0);
                if i < len - 1 { self.idea_state.select(Some(i + 1)); }
            }
            _ => {}
        }
    }

    fn enter(&mut self) {
        match &self.screen {
            Screen::ProjectList => {
                if let Some(i) = self.project_state.selected() {
                    self.prev_screen = Some(Screen::ProjectList);
                    self.screen = Screen::ProjectDetail(i);
                }
            }
            Screen::IdeaList => {
                if let Some(i) = self.idea_state.selected() {
                    self.prev_screen = Some(Screen::IdeaList);
                    self.screen = Screen::IdeaDetail(i);
                }
            }
            _ => {}
        }
    }
}

pub async fn run_tui() -> Result<()> {
    let cfg = Config::load()?;

    // ターミナルセットアップ
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(cfg.clone());

    // 初期データ取得（プロジェクト）
    let url = format!("{}/projects?limit=80&offset=0", cfg.api_base_url);
    match cached_get::<PaginatedResponse<ApiProject>>(&url, &cfg).await {
        Ok(resp) => {
            app.projects = resp.data;
            app.project_state.select(Some(0));
        }
        Err(e) => {
            // エラーは無視し、空リストのまま
            let _ = e;
        }
    }
    app.loading = false;

    // イベントループ
    loop {
        // 描画
        terminal.draw(|f| {
            let area = f.area();
            f.render_widget(
                ratatui::widgets::Block::default()
                    .style(ratatui::style::Style::default().bg(COLOR_BG)),
                area,
            );
            let (header_area, body_area, footer_area) = split_layout(area);

            match &app.screen {
                Screen::ProjectList => {
                    render_header(f, header_area, "プロジェクト");
                    if app.loading {
                        render_loading(f, body_area);
                    } else {
                        render_project_list(f, body_area, &app.projects, &mut app.project_state);
                    }
                    render_footer(f, footer_area, &[
                        ("Enter", "詳細"), ("i", "アイデア"), ("r", "再取得"), ("?", "ヘルプ"), ("q", "終了"),
                    ]);
                }
                Screen::ProjectDetail(idx) => {
                    let title = app.projects.get(*idx).map(|p| p.name.as_str()).unwrap_or("詳細");
                    render_header(f, header_area, title);
                    if let Some(project) = app.projects.get(*idx) {
                        render_project_detail(f, body_area, project);
                    }
                    render_footer(f, footer_area, &[("b", "戻る"), ("q", "終了")]);
                }
                Screen::IdeaList => {
                    render_header(f, header_area, "アイデア");
                    if app.loading {
                        render_loading(f, body_area);
                    } else {
                        render_idea_list(f, body_area, &app.ideas, &mut app.idea_state);
                    }
                    render_footer(f, footer_area, &[
                        ("Enter", "詳細"), ("p", "プロジェクト"), ("r", "再取得"), ("?", "ヘルプ"), ("q", "終了"),
                    ]);
                }
                Screen::IdeaDetail(idx) => {
                    let title = app.ideas.get(*idx).map(|i| i.title.as_str()).unwrap_or("詳細");
                    render_header(f, header_area, title);
                    if let Some(idea) = app.ideas.get(*idx) {
                        render_idea_detail(f, body_area, idea);
                    }
                    render_footer(f, footer_area, &[("b", "戻る"), ("q", "終了")]);
                }
                Screen::Help => {
                    render_header(f, header_area, "ヘルプ");
                    render_help(f, body_area);
                    render_footer(f, footer_area, &[("b / Esc", "戻る"), ("q", "終了")]);
                }
            }
        })?;

        // イベント処理
        if let Some(event) = poll_event()? {
            match event {
                AppEvent::Key(key) => {
                    if is_quit(&key) { break; }
                    match key.code {
                        KeyCode::Up   => app.move_up(),
                        KeyCode::Down => app.move_down(),
                        KeyCode::Enter => app.enter(),
                        KeyCode::Char('b') | KeyCode::Backspace => app.go_back(),
                        KeyCode::Char('p') => {
                            app.screen = Screen::ProjectList;
                        }
                        KeyCode::Char('i') => {
                            // アイデア未取得なら取得
                            if app.ideas.is_empty() {
                                let url = format!("{}/ideas?limit=80&offset=0", app.cfg.api_base_url);
                                let cfg_clone = app.cfg.clone();
                                if let Ok(resp) = cached_get::<PaginatedResponse<ApiIdea>>(&url, &cfg_clone).await {
                                    app.ideas = resp.data;
                                    app.idea_state.select(Some(0));
                                }
                            }
                            app.screen = Screen::IdeaList;
                        }
                        KeyCode::Char('r') => {
                            // キャッシュ無視で再取得
                            let url = match &app.screen {
                                Screen::ProjectList | Screen::ProjectDetail(_) => {
                                    format!("{}/projects?limit=80&offset=0", app.cfg.api_base_url)
                                }
                                Screen::IdeaList | Screen::IdeaDetail(_) => {
                                    format!("{}/ideas?limit=80&offset=0", app.cfg.api_base_url)
                                }
                                _ => continue,
                            };
                            // TTL=0 で強制再取得（キャッシュを上書き）
                            let mut no_cache_cfg = app.cfg.clone();
                            no_cache_cfg.cache_enabled = false;
                            match app.screen {
                                Screen::ProjectList | Screen::ProjectDetail(_) => {
                                    if let Ok(resp) = cached_get::<PaginatedResponse<ApiProject>>(&url, &no_cache_cfg).await {
                                        app.projects = resp.data;
                                        app.project_state.select(Some(0));
                                    }
                                }
                                Screen::IdeaList | Screen::IdeaDetail(_) => {
                                    if let Ok(resp) = cached_get::<PaginatedResponse<ApiIdea>>(&url, &no_cache_cfg).await {
                                        app.ideas = resp.data;
                                        app.idea_state.select(Some(0));
                                    }
                                }
                                _ => {}
                            }
                        }
                        KeyCode::Char('?') => {
                            app.prev_screen = Some(app.screen.clone());
                            app.screen = Screen::Help;
                        }
                        _ => {}
                    }
                }
                AppEvent::Tick => {}
            }
        }
    }

    // ターミナル復元
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}