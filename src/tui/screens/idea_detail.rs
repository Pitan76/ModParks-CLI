use ratatui::{Frame, layout::Rect};
use crate::tui::App;
use crate::tui::ui::{render_header, render_footer, render_idea_detail};

pub fn render(app: &App, f: &mut Frame, header_area: Rect, body_area: Rect, footer_area: Rect, idx: usize) {
    let title = app.ideas.get(idx).map(|i| i.title.as_str()).unwrap_or("詳細");
    render_header(f, header_area, title, None);
    if let Some(idea) = app.ideas.get(idx) {
        render_idea_detail(f, body_area, idea);
    }
    render_footer(f, footer_area, &[("b", "戻る"), ("q", "終了")]);
}
