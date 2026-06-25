use ratatui::{Frame, layout::Rect};
use crate::tui::App;
use crate::tui::ui::{render_header, render_footer, render_help};

pub fn render(_app: &App, f: &mut Frame, header_area: Rect, body_area: Rect, footer_area: Rect) {
    render_header(f, header_area, "ヘルプ", None);
    render_help(f, body_area);
    render_footer(f, footer_area, &[("b / Esc", "戻る"), ("q", "終了")]);
}
