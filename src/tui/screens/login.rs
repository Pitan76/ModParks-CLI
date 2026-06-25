use ratatui::{Frame, layout::{Constraint, Direction, Layout, Rect}};
use crate::tui::{App, LoginType, InputMode};
use crate::tui::ui::{render_header, render_footer, render_input, render_error, render_loading};

fn split_login(area: Rect) -> (Rect, Rect, Rect, Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Option Selector
            Constraint::Length(3), // ID / API Key
            Constraint::Length(3), // Password
            Constraint::Length(3), // TOTP
            Constraint::Min(0),    // Padding/Message
        ])
        .split(area);
    (chunks[0], chunks[1], chunks[2], chunks[3], chunks[4])
}

pub fn render(app: &App, f: &mut Frame, header_area: Rect, body_area: Rect, footer_area: Rect) {
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
    render_footer(f, footer_area, &[("Tab", "方式切替"), ("↑/↓", "移動"), ("Enter", "実行"), ("Esc", "キャンセル")]);
}
