// src/tui/handlers.rs
use crossterm::event::{Event, KeyCode};
use tui_input::backend::crossterm::EventHandler;

use crate::api_client::{cached_get, auth_login};
use crate::api_models::{ApiProject, ApiIdea};
use crate::pagination::PaginatedResponse;
use crate::tui::app::{App, Screen, InputMode, LoginType};
use crate::tui::events::is_quit;
use tui_input::Input;

pub async fn fetch_projects(app: &mut App) {
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

pub async fn fetch_my_projects(app: &mut App) {
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

pub async fn fetch_ideas(app: &mut App) {
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


pub async fn handle_key_event(app: &mut App, key: crossterm::event::KeyEvent) -> bool {
    if is_quit(&key) { return true; }

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
                                        fetch_projects(app).await;
                                    }
                                    InputMode::LoginId => {
                                        if app.login_type == LoginType::Password {
                                            app.input_mode = InputMode::LoginPassword;
                                        } else {
                                            // API Key Login 実行
                                            app.cfg.api_key = app.login_id_input.value().to_string();
                                            app.current_user = crate::api_client::auth_me(&app.cfg).await.ok();
                                            let _ = app.cfg.save();
                                            app.input_mode = InputMode::Normal;
                                            app.screen = Screen::ProjectList;
                                            fetch_projects(app).await;
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
                                                    app.current_user = crate::api_client::auth_me(&app.cfg).await.ok();
                                                    let _ = app.cfg.save();
                                                    app.input_mode = InputMode::Normal;
                                                    app.screen = Screen::ProjectList;
                                                    fetch_projects(app).await;
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
                                                    app.current_user = crate::api_client::auth_me(&app.cfg).await.ok();
                                                    let _ = app.cfg.save();
                                                    app.input_mode = InputMode::Normal;
                                                    app.screen = Screen::ProjectList;
                                                    fetch_projects(app).await;
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
                                            fetch_projects(app).await;
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
                        return false;
                    }

                    // 通常モードの処理
                    if app.error_msg.is_some() {
                        match key.code {
                            KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('b') => {
                                app.error_msg = None;
                            }
                            _ => {}
                        }
                        return false;
                    }

                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return true,
                        KeyCode::Up => app.move_up(),
                        KeyCode::Down => app.move_down(),
                        KeyCode::Right | KeyCode::Char('>') => {
                            if app.screen == Screen::ProjectList {
                                // APIの count が全件数ではなく現在の件数になっているため、
                                // 取得件数が limit と等しければ次のページがあるかもしれないと判断する
                                if app.project_total == app.limit {
                                    app.project_page += 1;
                                    fetch_projects(app).await;
                                }
                            } else if app.screen == Screen::MyProjects {
                                if app.my_project_total == app.limit {
                                    app.my_project_page += 1;
                                    fetch_my_projects(app).await;
                                }
                            } else if app.screen == Screen::IdeaList {
                                if app.idea_total == app.limit {
                                    app.idea_page += 1;
                                    fetch_ideas(app).await;
                                }
                            }
                        }
                        KeyCode::Left | KeyCode::Char('<') => {
                            if app.screen == Screen::ProjectList && app.project_page > 1 {
                                app.project_page -= 1;
                                fetch_projects(app).await;
                            } else if app.screen == Screen::MyProjects && app.my_project_page > 1 {
                                app.my_project_page -= 1;
                                fetch_my_projects(app).await;
                            } else if app.screen == Screen::IdeaList && app.idea_page > 1 {
                                app.idea_page -= 1;
                                fetch_ideas(app).await;
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
                                fetch_projects(app).await;
                            } else if app.screen == Screen::MyProjects {
                                fetch_my_projects(app).await;
                            } else if app.screen == Screen::IdeaList {
                                fetch_ideas(app).await;
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
                            if app.projects.is_empty() { fetch_projects(app).await; }
                        }
                        KeyCode::Char('m') => {
                            if app.current_user.is_some() {
                                app.screen = Screen::MyProjects;
                                if app.my_projects.is_empty() { fetch_my_projects(app).await; }
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
                            if app.ideas.is_empty() { fetch_ideas(app).await; }
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
                            if app.screen == Screen::ProjectList { fetch_projects(app).await; }
                            else if app.screen == Screen::MyProjects { fetch_my_projects(app).await; }
                            else if app.screen == Screen::IdeaList { fetch_ideas(app).await; }
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
    false
}

