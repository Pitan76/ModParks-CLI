use ratatui::{Frame, layout::Rect};
use crate::tui::{App};
use crate::tui::ui::{render_header, render_footer, render_profile};
use crate::api_models::Author;

pub fn render(app: &App, f: &mut Frame, header_area: Rect, body_area: Rect, footer_area: Rect, author_opt: Option<&Author>) {
    render_header(f, header_area, "プロフィール", None);
    render_profile(f, body_area, author_opt, app.current_user.as_ref(), &app.my_projects);
    render_footer(f, footer_area, &[("b", "戻る"), ("q", "終了")]);
}
