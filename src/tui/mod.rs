// src/tui/mod.rs
pub mod events;
pub mod ui;

use anyhow::Result;
use crossterm::{
    event::{Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, widgets::ListState, Terminal};
use std::io;
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::api_models::{ApiProject, ApiIdea};
use crate::api_client::{cached_get, auth_login};
use crate::config::Config;
use crate::pagination::PaginatedResponse;
use self::events::{poll_event, AppEvent, is_quit};
use self::ui::*;

#[derive(Debug, Clone, PartialEq)]
enum Screen {
    Login,
    ProjectList,
    ProjectDetail(usize),
    IdeaList,
    IdeaDetail(usize),
    Help,
}

#[derive(Debug, Clone, PartialEq)]
enum InputMode {
    Normal,
    Search,
    LoginId,
    LoginPassword,
    LoginTotp,
}

#[derive(Debug, Clone, PartialEq)]
enum LoginType {
    ApiKey,
    Password,
}

struct App {
    screen: Screen,
    prev_screen: Option<Screen>,
    input_mode: InputMode,
    
    projects: Vec<ApiProject>,
    ideas: Vec<ApiIdea>,
    project_state: ListState,
    idea_state: ListState,
    
    // Pagination & Search
    project_page: u32,
    idea_page: u32,
    limit: u32,
    search_input: Input,
    
    // Login
    login_type: LoginType,
    login_id_input: Input,
    login_pw_input: Input,
    login_totp_input: Input,
    login_requires_totp: bool,
    
    loading: bool,
    error_msg: Option<String>,
    cfg: Config,
}

impl App {
    fn new(cfg: Config) -> Self {
        let mut project_state = ListState::default();
        project_state.select(Some(0));
        let mut idea_state = ListState::default();
        idea_state.select(Some(0));
        
        let initial_screen = if cfg.api_key.is_empty() { Screen::Login } else { Screen::ProjectList };

        Self {
            screen: initial_screen,
            prev_screen: None,
            input_mode: InputMode::Normal,
            projects: vec![],
            ideas: vec![],
            project_state,
            idea_state,
            project_page: 1,
            idea_page: 1,
            limit: 20,
            search_input: Input::default(),
            login_type: LoginType::Password,
            login_id_input: Input::default(),
            login_pw_input: Input::default(),
            login_totp_input: Input::default(),
            login_requires_totp: false,
            loading: true,
            error_msg: None,
            cfg,
        }
    }

    // （以下各種ヘルパーは省略せずに記述）
    fn go_back(&mut self) {
        if self.input_mode != InputMode::Normal {
            self.input_mode = InputMode::Normal;
            return;
        }
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
            Screen::Login => {
                if self.input_mode == InputMode::LoginPassword {
                    self.input_mode = InputMode::LoginId;
                } else if self.input_mode == InputMode::LoginTotp {
                    self.input_mode = InputMode::LoginPassword;
                }
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
            Screen::Login => {
                if self.input_mode == InputMode::LoginId {
                    self.input_mode = InputMode::LoginPassword;
                } else if self.input_mode == InputMode::LoginPassword && self.login_requires_totp {
                    self.input_mode = InputMode::LoginTotp;
                }
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

async fn fetch_projects(app: &mut App) {
    app.loading = true;
    app.error_msg = None;
    let offset = (app.project_page.saturating_sub(1)) * app.limit;
    let q = app.search_input.value();
    let q_param = if q.is_empty() { "".to_string() } else { format!("&q={}", q) };
    let url = format!("{}/projects?limit={}&offset={}{}", app.cfg.api_base_url, app.limit, offset, q_param);
    
    match cached_get::<PaginatedResponse<ApiProject>>(&url, &app.cfg).await {
        Ok(resp) => {
            app.projects = resp.data;
            app.project_state.select(Some(0));
        }
        Err(e) => {
            app.error_msg = Some(e.to_string());
        }
    }
    app.loading = false;
}

async fn fetch_ideas(app: &mut App) {
    app.loading = true;
    app.error_msg = None;
    let offset = (app.idea_page.saturating_sub(1)) * app.limit;
    let url = format!("{}/ideas?limit={}&offset={}", app.cfg.api_base_url, app.limit, offset);
    
    match cached_get::<PaginatedResponse<ApiIdea>>(&url, &app.cfg).await {
        Ok(resp) => {
            app.ideas = resp.data;
            app.idea_state.select(Some(0));
        }
        Err(e) => {
            app.error_msg = Some(e.to_string());
        }
    }
    app.loading = false;
}

pub async fn run_tui() -> Result<()> {
    let cfg = Config::load()?;
    let mut app = App::new(cfg);

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

            match &app.screen {
                Screen::Login => {
                    render_header(f, header_area, "ログイン", None);
                    
                    let (type_area, id_area, pw_area, totp_area, msg_area) = split_login(body_area);
                    
                    let type_txt = if app.login_type == LoginType::Password { "[*] ID/Password   [ ] API Key" } else { "[ ] ID/Password   [*] API Key" };
                    f.render_widget(ratatui::widgets::Paragraph::new(type_txt).block(ratatui::widgets::Block::default().borders(ratatui::widgets::Borders::ALL).title(" ログイン方式 (Tabで切替) ")), type_area);
                    
                    if app.login_type == LoginType::Password {
                        render_input(f, id_area, "ID / Email (Enterで次へ)", &app.login_id_input, false);
                        render_input(f, pw_area, "Password (Enterでログイン)", &app.login_pw_input, true);
                        if app.login_requires_totp {
                            render_input(f, totp_area, "2FA Code (Enterでログイン)", &app.login_totp_input, false);
                        }
                    } else {
                        render_input(f, id_area, "API Key (Enterでログイン)", &app.login_id_input, true);
                    }
                    
                    if let Some(err) = &app.error_msg {
                        render_error(f, msg_area, err);
                    } else if app.loading {
                        render_loading(f, msg_area);
                    }
                    render_footer(f, footer_area, &[("Tab", "方式切替"), ("↑/↓", "移動"), ("Enter", "実行"), ("q", "終了")]);
                }
                Screen::ProjectList => {
                    let page_str = format!("{}件表示 | {}ページ", app.limit, app.project_page);
                    render_header(f, header_area, "プロジェクト", Some(&page_str));
                    let (search_area, list_area) = split_search(body_area);
                    render_input(f, search_area, "検索 (/ で入力)", &app.search_input, false);
                    
                    if let Some(err) = &app.error_msg {
                        render_error(f, list_area, err);
                    } else if app.loading {
                        render_loading(f, list_area);
                    } else {
                        render_project_list(f, list_area, &app.projects, &mut app.project_state);
                    }
                    render_footer(f, footer_area, &[
                        ("Enter", "詳細"), ("< / >", "ページ遷移"), ("l", "件数変更"), ("i", "アイデア"), ("/", "検索"), ("r", "再取得"), ("?", "ヘルプ"), ("q", "終了"),
                    ]);
                }
                Screen::ProjectDetail(idx) => {
                    let title = app.projects.get(*idx).map(|p| p.name.as_str()).unwrap_or("詳細");
                    render_header(f, header_area, title, None);
                    if let Some(project) = app.projects.get(*idx) {
                        render_project_detail(f, body_area, project);
                    }
                    render_footer(f, footer_area, &[("b", "戻る"), ("q", "終了")]);
                }
                Screen::IdeaList => {
                    let page_str = format!("{}件表示 | {}ページ", app.limit, app.idea_page);
                    render_header(f, header_area, "アイデア", Some(&page_str));
                    if let Some(err) = &app.error_msg {
                        render_error(f, body_area, err);
                    } else if app.loading {
                        render_loading(f, body_area);
                    } else {
                        render_idea_list(f, body_area, &app.ideas, &mut app.idea_state);
                    }
                    render_footer(f, footer_area, &[
                        ("Enter", "詳細"), ("< / >", "ページ遷移"), ("p", "プロジェクト"), ("r", "再取得"), ("?", "ヘルプ"), ("q", "終了"),
                    ]);
                }
                Screen::IdeaDetail(idx) => {
                    let title = app.ideas.get(*idx).map(|i| i.title.as_str()).unwrap_or("詳細");
                    render_header(f, header_area, title, None);
                    if let Some(idea) = app.ideas.get(*idx) {
                        render_idea_detail(f, body_area, idea);
                    }
                    render_footer(f, footer_area, &[("b", "戻る"), ("q", "終了")]);
                }
                Screen::Help => {
                    render_header(f, header_area, "ヘルプ", None);
                    render_help(f, body_area);
                    render_footer(f, footer_area, &[("b / Esc", "戻る"), ("q", "終了")]);
                }
            }
        })?;

        if let Some(app_event) = poll_event()? {
            match app_event {
                AppEvent::Key(key) => {
                    if is_quit(&key) { break; }

                    // 入力モードの処理
                    if app.input_mode != InputMode::Normal {
                        match key.code {
                            KeyCode::Esc => { app.input_mode = InputMode::Normal; }
                            KeyCode::Enter => {
                                match app.input_mode {
                                    InputMode::Search => {
                                        app.input_mode = InputMode::Normal;
                                        app.project_page = 1;
                                        fetch_projects(&mut app).await;
                                    }
                                    InputMode::LoginId => {
                                        if app.login_type == LoginType::Password {
                                            app.input_mode = InputMode::LoginPassword;
                                        } else {
                                            // API Key Login 実行
                                            app.cfg.api_key = app.login_id_input.value().to_string();
                                            let _ = app.cfg.save();
                                            app.input_mode = InputMode::Normal;
                                            app.screen = Screen::ProjectList;
                                            fetch_projects(&mut app).await;
                                        }
                                    }
                                    InputMode::LoginPassword => {
                                        // Login API 実行
                                        app.loading = true;
                                        app.error_msg = None;
                                        let id = app.login_id_input.value().to_string();
                                        let pw = app.login_pw_input.value().to_string();
                                        match auth_login(&app.cfg, &id, &pw, None).await {
                                            Ok(res) => {
                                                if res.requires_2fa.unwrap_or(false) {
                                                    app.login_requires_totp = true;
                                                    app.input_mode = InputMode::LoginTotp;
                                                } else if let Some(k) = res.api_key {
                                                    app.cfg.api_key = k;
                                                    let _ = app.cfg.save();
                                                    app.input_mode = InputMode::Normal;
                                                    app.screen = Screen::ProjectList;
                                                    fetch_projects(&mut app).await;
                                                }
                                            }
                                            Err(e) => {
                                                app.error_msg = Some(e.to_string());
                                            }
                                        }
                                        app.loading = false;
                                    }
                                    InputMode::LoginTotp => {
                                        // Login API 実行 (with TOTP)
                                        app.loading = true;
                                        app.error_msg = None;
                                        let id = app.login_id_input.value().to_string();
                                        let pw = app.login_pw_input.value().to_string();
                                        let totp = app.login_totp_input.value().to_string();
                                        match auth_login(&app.cfg, &id, &pw, Some(&totp)).await {
                                            Ok(res) => {
                                                if let Some(k) = res.api_key {
                                                    app.cfg.api_key = k;
                                                    let _ = app.cfg.save();
                                                    app.input_mode = InputMode::Normal;
                                                    app.screen = Screen::ProjectList;
                                                    fetch_projects(&mut app).await;
                                                } else {
                                                    app.error_msg = Some("2FAに失敗しました".into());
                                                }
                                            }
                                            Err(e) => {
                                                app.error_msg = Some(e.to_string());
                                            }
                                        }
                                        app.loading = false;
                                    }
                                    _ => {}
                                }
                            }
                            _ => {
                                let req = Event::Key(key);
                                match app.input_mode {
                                    InputMode::Search => { app.search_input.handle_event(&req); }
                                    InputMode::LoginId => { app.login_id_input.handle_event(&req); }
                                    InputMode::LoginPassword => { app.login_pw_input.handle_event(&req); }
                                    InputMode::LoginTotp => { app.login_totp_input.handle_event(&req); }
                                    _ => {}
                                }
                            }
                        }
                        continue;
                    }

                    // 通常モードの処理
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Up => app.move_up(),
                        KeyCode::Down => app.move_down(),
                        KeyCode::Right | KeyCode::Char('>') => {
                            if app.screen == Screen::ProjectList {
                                app.project_page += 1;
                                fetch_projects(&mut app).await;
                            } else if app.screen == Screen::IdeaList {
                                app.idea_page += 1;
                                fetch_ideas(&mut app).await;
                            }
                        }
                        KeyCode::Left | KeyCode::Char('<') => {
                            if app.screen == Screen::ProjectList && app.project_page > 1 {
                                app.project_page -= 1;
                                fetch_projects(&mut app).await;
                            } else if app.screen == Screen::IdeaList && app.idea_page > 1 {
                                app.idea_page -= 1;
                                fetch_ideas(&mut app).await;
                            }
                        }
                        KeyCode::Enter => app.enter(),
                        KeyCode::Char('b') | KeyCode::Backspace => app.go_back(),
                        KeyCode::Char('/') => {
                            if app.screen == Screen::ProjectList {
                                app.input_mode = InputMode::Search;
                            }
                        }
                        KeyCode::Char('l') => {
                            // 件数変更 20 -> 40 -> 80 -> 20
                            app.limit = match app.limit {
                                20 => 40,
                                40 => 80,
                                _ => 20,
                            };
                            if app.screen == Screen::ProjectList {
                                fetch_projects(&mut app).await;
                            } else if app.screen == Screen::IdeaList {
                                fetch_ideas(&mut app).await;
                            }
                        }
                        KeyCode::Char('L') => {
                            // ログアウト
                            app.cfg.api_key = String::new();
                            let _ = app.cfg.save();
                            app.screen = Screen::Login;
                        }
                        KeyCode::Char('p') => {
                            app.screen = Screen::ProjectList;
                            if app.projects.is_empty() { fetch_projects(&mut app).await; }
                        }
                        KeyCode::Char('i') => {
                            app.screen = Screen::IdeaList;
                            if app.ideas.is_empty() { fetch_ideas(&mut app).await; }
                        }
                        KeyCode::Tab => {
                            if app.screen == Screen::Login {
                                app.login_type = if app.login_type == LoginType::Password { LoginType::ApiKey } else { LoginType::Password };
                            }
                        }
                        KeyCode::Char('?') => {
                            app.prev_screen = Some(app.screen.clone());
                            app.screen = Screen::Help;
                        }
                        KeyCode::Char('r') => {
                            app.cfg.cache_enabled = false; // 一時的に無効
                            if app.screen == Screen::ProjectList { fetch_projects(&mut app).await; }
                            else if app.screen == Screen::IdeaList { fetch_ideas(&mut app).await; }
                            app.cfg.cache_enabled = true;
                        }
                        _ => {}
                    }
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