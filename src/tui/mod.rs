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
    ProjectCreate,
    ProjectEdit(String),
    IdeaList,
    IdeaDetail(usize),
    Help,
    Profile,
    MyProjects,
    MyProjectDetail(usize),
}

#[derive(Debug, Clone, PartialEq)]
enum InputMode {
    Normal,
    Search,
    LoginId,
    LoginPassword,
    LoginTotp,
    ProjectForm(usize),
    DownloadForm,
    UploadForm(usize), // file_path, version_number, changelog など
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
    my_projects: Vec<ApiProject>,
    ideas: Vec<ApiIdea>,
    project_state: ListState,
    my_project_state: ListState,
    idea_state: ListState,
    
    // Pagination & Search
    project_page: u32,
    project_total: u32,
    my_project_page: u32,
    my_project_total: u32,
    idea_page: u32,
    idea_total: u32,
    limit: u32,
    search_input: Input,
    
    // Login
    login_type: LoginType,
    login_id_input: Input,
    login_pw_input: Input,
    login_totp_input: Input,
    login_requires_totp: bool,
    
    project_form_inputs: Vec<Input>,
    download_input: Input,
    upload_inputs: Vec<Input>,

    current_user: Option<crate::api_models::AuthMe>,

    loading: bool,
    error_msg: Option<String>,
    cfg: Config,
}

impl App {
    fn new(cfg: Config) -> Self {
        let mut project_state = ListState::default();
        project_state.select(Some(0));
        let mut my_project_state = ListState::default();
        my_project_state.select(Some(0));
        let mut idea_state = ListState::default();
        idea_state.select(Some(0));
        
        let initial_screen = if cfg.api_key.is_empty() { Screen::Login } else { Screen::ProjectList };
        let initial_input = if cfg.api_key.is_empty() { InputMode::LoginId } else { InputMode::Normal };

        Self {
            screen: initial_screen,
            prev_screen: None,
            input_mode: initial_input,
            projects: vec![],
            my_projects: vec![],
            ideas: vec![],
            project_state,
            my_project_state,
            idea_state,
            project_page: 1,
            project_total: 0,
            my_project_page: 1,
            my_project_total: 0,
            idea_page: 1,
            idea_total: 0,
            limit: 20,
            search_input: Input::default(),
            login_type: LoginType::Password,
            login_id_input: Input::default(),
            login_pw_input: Input::default(),
            login_totp_input: Input::default(),
            login_requires_totp: false,
            project_form_inputs: vec![Input::default(); 4],
            download_input: Input::default().with_value(std::env::current_dir().unwrap_or_default().to_string_lossy().into_owned()),
            upload_inputs: vec![Input::default(); 6], // file/url, filename, version, loaders, mc_versions, changelog
            current_user: None,
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
                Screen::MyProjectDetail(_) => { self.screen = Screen::MyProjects; }
                Screen::IdeaDetail(_)    => { self.screen = Screen::IdeaList; }
                Screen::Help             | Screen::ProjectCreate | Screen::ProjectEdit(_) | Screen::Profile | Screen::MyProjects => { self.screen = Screen::ProjectList; }
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
            Screen::MyProjects => {
                let i = self.my_project_state.selected().unwrap_or(0);
                if i > 0 { self.my_project_state.select(Some(i - 1)); }
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
            Screen::MyProjects => {
                let len = self.my_projects.len();
                if len == 0 { return; }
                let i = self.my_project_state.selected().unwrap_or(0);
                if i < len - 1 { self.my_project_state.select(Some(i + 1)); }
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
            Screen::MyProjects => {
                if let Some(i) = self.my_project_state.selected() {
                    self.prev_screen = Some(Screen::MyProjects);
                    self.screen = Screen::MyProjectDetail(i);
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
            app.project_total = resp.meta.count as u32;
            app.project_state.select(Some(0));
        }
        Err(e) => {
            app.error_msg = Some(e.to_string());
        }
    }
    app.loading = false;
}

async fn fetch_my_projects(app: &mut App) {
    if let Some(me) = &app.current_user {
        app.loading = true;
        app.error_msg = None;
        let offset = (app.my_project_page.saturating_sub(1)) * app.limit;
        let url = format!("{}/projects?limit={}&offset={}&author={}", app.cfg.api_base_url, app.limit, offset, me.username);
        
        match cached_get::<PaginatedResponse<ApiProject>>(&url, &app.cfg).await {
            Ok(resp) => {
                app.my_projects = resp.data;
                app.my_project_total = resp.meta.count as u32;
                app.my_project_state.select(Some(0));
            }
            Err(e) => {
                app.error_msg = Some(e.to_string());
            }
        }
        app.loading = false;
    } else {
        app.error_msg = Some("ログインしていません".into());
    }
}

async fn fetch_ideas(app: &mut App) {
    app.loading = true;
    app.error_msg = None;
    let offset = (app.idea_page.saturating_sub(1)) * app.limit;
    let url = format!("{}/ideas?limit={}&offset={}", app.cfg.api_base_url, app.limit, offset);
    
    match cached_get::<PaginatedResponse<ApiIdea>>(&url, &app.cfg).await {
        Ok(resp) => {
            app.ideas = resp.data;
            app.idea_total = resp.meta.count as u32;
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

            match &app.screen {
                Screen::Login => {
                    render_header(f, header_area, "ログイン", None);
                    
                    let (type_area, id_area, pw_area, totp_area, msg_area) = split_login(body_area);
                    
                    let type_txt = if app.login_type == LoginType::Password { "[*] ID/Password   [ ] API Key" } else { "[ ] ID/Password   [*] API Key" };
                    f.render_widget(ratatui::widgets::Paragraph::new(type_txt).block(ratatui::widgets::Block::default().borders(ratatui::widgets::Borders::ALL).title(" ログイン方式 (Tabで切替) ")), type_area);
                    
                    if app.login_type == LoginType::Password {
                        render_input(f, id_area, "ID / Email (Enterで次へ)", &app.login_id_input, false, app.input_mode == InputMode::LoginId);
                        render_input(f, pw_area, "Password (Enterでログイン)", &app.login_pw_input, true, app.input_mode == InputMode::LoginPassword);
                        if app.login_requires_totp {
                            render_input(f, totp_area, "2FA Code (Enterでログイン)", &app.login_totp_input, false, app.input_mode == InputMode::LoginTotp);
                        }
                    } else {
                        render_input(f, id_area, "API Key (Enterでログイン)", &app.login_id_input, true, app.input_mode == InputMode::LoginId);
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
                    render_input(f, search_area, "検索 (/ で入力)", &app.search_input, false, app.input_mode == InputMode::Search);
                    
                    if let Some(err) = &app.error_msg {
                        render_error(f, list_area, err);
                    } else if app.loading {
                        render_loading(f, list_area);
                    } else {
                        render_project_list(f, list_area, &app.projects, &mut app.project_state);
                    }
                    render_footer(f, footer_area, &[
                        ("Enter", "詳細"), ("< / >", "ページ遷移"), ("l", "件数変更"), ("i", "アイデア"), ("m", "自分のプロジェクト"), ("u", "プロフィール"), ("/", "検索"), ("r", "再取得"), ("?", "ヘルプ"), ("q", "終了"),
                    ]);
                }
                Screen::MyProjects => {
                    let page_str = format!("{}件表示 | {}ページ", app.limit, app.my_project_page);
                    render_header(f, header_area, "自分のプロジェクト", Some(&page_str));
                    
                    if let Some(err) = &app.error_msg {
                        render_error(f, body_area, err);
                    } else if app.loading {
                        render_loading(f, body_area);
                    } else {
                        render_project_list(f, body_area, &app.my_projects, &mut app.my_project_state);
                    }
                    render_footer(f, footer_area, &[
                        ("Enter", "詳細"), ("< / >", "ページ遷移"), ("p", "全体プロジェクト"), ("b", "戻る"), ("q", "終了"),
                    ]);
                }
                Screen::Profile => {
                    render_header(f, header_area, "プロフィール", None);
                    render_profile(f, body_area, app.current_user.as_ref());
                    render_footer(f, footer_area, &[("b", "戻る"), ("q", "終了")]);
                }
                Screen::ProjectDetail(idx) | Screen::MyProjectDetail(idx) => {
                    let (title, project) = if matches!(app.screen, Screen::ProjectDetail(_)) {
                        (app.projects.get(*idx).map(|p| p.name.as_str()).unwrap_or("詳細"), app.projects.get(*idx))
                    } else {
                        (app.my_projects.get(*idx).map(|p| p.name.as_str()).unwrap_or("詳細"), app.my_projects.get(*idx))
                    };
                    render_header(f, header_area, title, None);

                    match app.input_mode {
                        InputMode::DownloadForm => {
                            let (path_a, msg_a) = split_download_form(body_area);
                            render_input(f, path_a, "保存先ディレクトリパス", &app.download_input, false, true);
                            if let Some(err) = &app.error_msg { render_error(f, msg_a, err); }
                            else if app.loading { render_loading(f, msg_a); }
                            render_footer(f, footer_area, &[("Enter", "ダウンロード実行"), ("Esc", "キャンセル")]);
                        }
                        InputMode::UploadForm(focus) => {
                            let (file_a, name_a, ver_a, load_a, mc_a, change_a, msg_a) = split_upload_form(body_area);
                            render_input(f, file_a, "File Path or URL", &app.upload_inputs[0], false, focus == 0);
                            render_input(f, name_a, "File Name (optional for local file)", &app.upload_inputs[1], false, focus == 1);
                            render_input(f, ver_a, "Version Number", &app.upload_inputs[2], false, focus == 2);
                            render_input(f, load_a, "Loaders (comma separated)", &app.upload_inputs[3], false, focus == 3);
                            render_input(f, mc_a, "Minecraft Versions (comma separated)", &app.upload_inputs[4], false, focus == 4);
                            render_input(f, change_a, "Changelog", &app.upload_inputs[5], false, focus == 5);
                            if let Some(err) = &app.error_msg { render_error(f, msg_a, err); }
                            else if app.loading { render_loading(f, msg_a); }
                            render_footer(f, footer_area, &[("Tab", "項目移動"), ("Enter", "アップロード実行"), ("Esc", "キャンセル")]);
                        }
                        _ => {
                            if let Some(p) = project {
                                render_project_detail(f, body_area, p);
                            }
                            if let Some(err) = &app.error_msg {
                                // エラーメッセージを画面中央に被せる簡易表示
                                let err_area = ratatui::layout::Rect::new(area.x + 5, area.y + 5, area.width.saturating_sub(10), 3);
                                f.render_widget(ratatui::widgets::Clear, err_area);
                                render_error(f, err_area, err);
                            } else if app.loading {
                                let load_area = ratatui::layout::Rect::new(area.x + 5, area.y + 5, area.width.saturating_sub(10), 3);
                                f.render_widget(ratatui::widgets::Clear, load_area);
                                render_loading(f, load_area);
                            }
                            render_footer(f, footer_area, &[("b", "戻る"), ("e", "編集"), ("d", "DL"), ("v", "UP"), ("q", "終了")]);
                        }
                    }
                }
                Screen::ProjectCreate | Screen::ProjectEdit(_) => {
                    let title = if matches!(app.screen, Screen::ProjectCreate) { "プロジェクト作成" } else { "プロジェクト編集" };
                    render_header(f, header_area, title, None);
                    let (name_a, slug_a, desc_a, type_a, msg_a) = split_project_form(body_area);
                    
                    let focus = if let InputMode::ProjectForm(i) = app.input_mode { i } else { 99 };
                    
                    render_input(f, name_a, "Name", &app.project_form_inputs[0], false, focus == 0);
                    render_input(f, slug_a, "Slug", &app.project_form_inputs[1], false, focus == 1);
                    render_input(f, desc_a, "Description", &app.project_form_inputs[2], false, focus == 2);
                    render_input(f, type_a, "Type (mod/plugin)", &app.project_form_inputs[3], false, focus == 3);

                    if let Some(err) = &app.error_msg {
                        render_error(f, msg_a, err);
                    } else if app.loading {
                        render_loading(f, msg_a);
                    }
                    render_footer(f, footer_area, &[("Tab", "項目移動"), ("Enter", "保存"), ("Esc", "キャンセル")]);
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
                            KeyCode::Tab => {
                                if let InputMode::ProjectForm(focus) = app.input_mode {
                                    app.input_mode = InputMode::ProjectForm((focus + 1) % 4);
                                } else if let InputMode::UploadForm(focus) = app.input_mode {
                                    app.input_mode = InputMode::UploadForm((focus + 1) % 6);
                                }
                            }
                            KeyCode::BackTab => {
                                if let InputMode::ProjectForm(focus) = app.input_mode {
                                    app.input_mode = InputMode::ProjectForm((focus + 3) % 4);
                                } else if let InputMode::UploadForm(focus) = app.input_mode {
                                    app.input_mode = InputMode::UploadForm((focus + 5) % 6);
                                }
                            }
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
                                    InputMode::ProjectForm(_) => {
                                        app.loading = true;
                                        app.error_msg = None;
                                        let name = app.project_form_inputs[0].value().to_string();
                                        let slug = app.project_form_inputs[1].value().to_string();
                                        let description = app.project_form_inputs[2].value().to_string();
                                        let p_type = app.project_form_inputs[3].value().to_string();
                                        let p_type = if p_type.is_empty() { "mod".to_string() } else { p_type };
                                        
                                        let mut is_success = false;
                                        if let Screen::ProjectCreate = app.screen {
                                            let req = crate::api_models::CreateProjectReq { name, slug, description, project_type: p_type };
                                            match crate::api_client::create_project(&app.cfg, &req).await {
                                                Ok(_) => is_success = true,
                                                Err(e) => app.error_msg = Some(e.to_string()),
                                            }
                                        } else if let Screen::ProjectEdit(ref orig_slug) = app.screen {
                                            let req = crate::api_models::UpdateProjectReq { 
                                                name: Some(name), 
                                                slug: Some(slug), 
                                                description: Some(description), 
                                                project_type: Some(p_type) 
                                            };
                                            match crate::api_client::update_project(&app.cfg, orig_slug, &req).await {
                                                Ok(_) => is_success = true,
                                                Err(e) => app.error_msg = Some(e.to_string()),
                                            }
                                        }
                                        
                                        if is_success {
                                            app.input_mode = InputMode::Normal;
                                            app.screen = Screen::ProjectList;
                                            fetch_projects(&mut app).await;
                                        }
                                        app.loading = false;
                                    }
                                    InputMode::DownloadForm => {
                                        app.loading = true;
                                        app.error_msg = None;
                                        let dir = app.download_input.value().to_string();
                                        
                                        // プロジェクト情報の取得
                                        let slug = if let Screen::ProjectDetail(idx) = app.screen {
                                            app.projects.get(idx).map(|p| p.slug.clone())
                                        } else if let Screen::MyProjectDetail(idx) = app.screen {
                                            app.my_projects.get(idx).map(|p| p.slug.clone())
                                        } else { None };

                                        if let Some(slug) = slug {
                                            let dir_path = if dir.is_empty() { None } else { Some(dir) };
                                            // ダウンロードの実行
                                            if let Err(e) = crate::commands::download::download_version(slug.clone(), None, dir_path).await {
                                                app.error_msg = Some(format!("ダウンロード失敗: {}", e));
                                            } else {
                                                app.input_mode = InputMode::Normal;
                                            }
                                        }
                                        app.loading = false;
                                    }
                                    InputMode::UploadForm(_) => {
                                        app.loading = true;
                                        app.error_msg = None;
                                        
                                        let slug = if let Screen::ProjectDetail(idx) = app.screen {
                                            app.projects.get(idx).map(|p| p.slug.clone())
                                        } else if let Screen::MyProjectDetail(idx) = app.screen {
                                            app.my_projects.get(idx).map(|p| p.slug.clone())
                                        } else { None };

                                        if let Some(slug) = slug {
                                            let file_path = app.upload_inputs[0].value().to_string();
                                            let file_name = app.upload_inputs[1].value().to_string();
                                            let version_num = app.upload_inputs[2].value().to_string();
                                            let loaders: Vec<String> = app.upload_inputs[3].value().split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                                            let mc_versions: Vec<String> = app.upload_inputs[4].value().split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                                            let changelog = app.upload_inputs[5].value().to_string();
                                            
                                            let file_opt = if file_path.starts_with("http") { None } else { Some(file_path.clone()) };
                                            let url_opt = if file_path.starts_with("http") { Some(file_path) } else { None };
                                            let file_name_opt = if file_name.is_empty() { None } else { Some(file_name) };
                                            let changelog_opt = if changelog.is_empty() { None } else { Some(changelog) };

                                            if let Err(e) = crate::commands::version::create_version(slug.clone(), file_opt, url_opt, file_name_opt, version_num, loaders, mc_versions, changelog_opt).await {
                                                app.error_msg = Some(format!("アップロード失敗: {}", e));
                                            } else {
                                                app.input_mode = InputMode::Normal;
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
                                    InputMode::ProjectForm(focus) => { app.project_form_inputs[focus].handle_event(&req); }
                                    InputMode::DownloadForm => { app.download_input.handle_event(&req); }
                                    InputMode::UploadForm(focus) => { app.upload_inputs[focus].handle_event(&req); }
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
                                // APIの count が全件数ではなく現在の件数になっているため、
                                // 取得件数が limit と等しければ次のページがあるかもしれないと判断する
                                if app.project_total == app.limit {
                                    app.project_page += 1;
                                    fetch_projects(&mut app).await;
                                }
                            } else if app.screen == Screen::MyProjects {
                                if app.my_project_total == app.limit {
                                    app.my_project_page += 1;
                                    fetch_my_projects(&mut app).await;
                                }
                            } else if app.screen == Screen::IdeaList {
                                if app.idea_total == app.limit {
                                    app.idea_page += 1;
                                    fetch_ideas(&mut app).await;
                                }
                            }
                        }
                        KeyCode::Left | KeyCode::Char('<') => {
                            if app.screen == Screen::ProjectList && app.project_page > 1 {
                                app.project_page -= 1;
                                fetch_projects(&mut app).await;
                            } else if app.screen == Screen::MyProjects && app.my_project_page > 1 {
                                app.my_project_page -= 1;
                                fetch_my_projects(&mut app).await;
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
                            } else if app.screen == Screen::MyProjects {
                                fetch_my_projects(&mut app).await;
                            } else if app.screen == Screen::IdeaList {
                                fetch_ideas(&mut app).await;
                            }
                        }
                        KeyCode::Char('L') => {
                            // ログアウト
                            app.cfg.api_key = String::new();
                            let _ = app.cfg.save();
                            app.screen = Screen::Login;
                            app.input_mode = InputMode::LoginId;
                        }
                        KeyCode::Char('p') => {
                            app.screen = Screen::ProjectList;
                            if app.projects.is_empty() { fetch_projects(&mut app).await; }
                        }
                        KeyCode::Char('m') => {
                            if app.current_user.is_some() {
                                app.screen = Screen::MyProjects;
                                if app.my_projects.is_empty() { fetch_my_projects(&mut app).await; }
                            } else {
                                app.error_msg = Some("ログインが必要です".into());
                            }
                        }
                        KeyCode::Char('u') => {
                            if app.current_user.is_some() {
                                app.prev_screen = Some(app.screen.clone());
                                app.screen = Screen::Profile;
                            } else {
                                app.error_msg = Some("ログインが必要です".into());
                            }
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
                            else if app.screen == Screen::MyProjects { fetch_my_projects(&mut app).await; }
                            else if app.screen == Screen::IdeaList { fetch_ideas(&mut app).await; }
                            app.cfg.cache_enabled = true;
                        }
                        KeyCode::Char('d') => {
                            if matches!(app.screen, Screen::ProjectDetail(_) | Screen::MyProjectDetail(_)) {
                                app.input_mode = InputMode::DownloadForm;
                            }
                        }
                        KeyCode::Char('v') => {
                            if matches!(app.screen, Screen::ProjectDetail(_) | Screen::MyProjectDetail(_)) {
                                app.upload_inputs = vec![Input::default(); 6];
                                app.input_mode = InputMode::UploadForm(0);
                            }
                        }
                        KeyCode::Char('c') => {
                            if app.screen == Screen::ProjectList {
                                app.prev_screen = Some(app.screen.clone());
                                app.screen = Screen::ProjectCreate;
                                app.project_form_inputs = vec![Input::default(); 4];
                                app.input_mode = InputMode::ProjectForm(0);
                            }
                        }
                        KeyCode::Char('e') => {
                            if let Screen::ProjectDetail(idx) = app.screen {
                                if let Some(p) = app.projects.get(idx) {
                                    let mut can_edit = false;
                                    if let Some(ref me) = app.current_user {
                                        if me.role == "admin" {
                                            can_edit = true;
                                        } else if let Some(ref author) = p.author {
                                            if author.username == me.username {
                                                can_edit = true;
                                            }
                                        }
                                    }
                                    
                                    if can_edit {
                                        app.prev_screen = Some(app.screen.clone());
                                        app.screen = Screen::ProjectEdit(p.slug.clone());
                                        app.project_form_inputs = vec![
                                            Input::default().with_value(p.name.clone()),
                                            Input::default().with_value(p.slug.clone()),
                                            Input::default().with_value(p.description.clone().unwrap_or_default()),
                                            Input::default().with_value(p.project_type.clone()),
                                        ];
                                        app.input_mode = InputMode::ProjectForm(0);
                                    } else {
                                        app.error_msg = Some("編集権限がありません".into());
                                    }
                                }
                            }
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