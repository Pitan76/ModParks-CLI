// src/tui/app.rs
use ratatui::widgets::ListState;
use tui_input::Input;
use crate::api_models::{ApiProject, ApiIdea, AuthMe};
use crate::config::Config;

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
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
pub enum InputMode {
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
pub enum LoginType {
    ApiKey,
    Password,
}

pub struct App {
    pub screen: Screen,
    pub prev_screen: Option<Screen>,
    pub input_mode: InputMode,
    
    pub projects: Vec<ApiProject>,
    pub my_projects: Vec<ApiProject>,
    pub ideas: Vec<ApiIdea>,
    pub project_state: ListState,
    pub my_project_state: ListState,
    pub idea_state: ListState,
    
    // Pagination & Search
    pub project_page: u32,
    pub project_total: u32,
    pub my_project_page: u32,
    pub my_project_total: u32,
    pub idea_page: u32,
    pub idea_total: u32,
    pub limit: u32,
    pub search_input: Input,
    
    // Login
    pub login_type: LoginType,
    pub login_id_input: Input,
    pub login_pw_input: Input,
    pub login_totp_input: Input,
    pub login_requires_totp: bool,
    
    pub project_form_inputs: Vec<Input>,
    pub download_input: Input,
    pub upload_inputs: Vec<Input>,

    pub current_user: Option<AuthMe>,

    pub loading: bool,
    pub error_msg: Option<String>,
    pub cfg: Config,
}

impl App {
    pub fn new(cfg: Config) -> Self {
        let mut project_state = ListState::default();
        project_state.select(Some(0));
        let mut my_project_state = ListState::default();
        my_project_state.select(Some(0));
        let mut idea_state = ListState::default();
        idea_state.select(Some(0));
        
        let initial_screen = Screen::ProjectList;
        let initial_input = InputMode::Normal;

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
    pub fn go_back(&mut self) {
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

    pub fn move_up(&mut self) {
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

    pub fn move_down(&mut self) {
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

    pub fn enter(&mut self) {
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

