use ratatui::{
    Frame, layout::Rect,
    style::Style,
    widgets::{Block, Borders, Paragraph, Wrap},
};
use crate::tui::App;
use crate::tui::ui::{render_header, render_footer, pad_width, COLOR_ACCENT, COLOR_FG};
use crate::api_models::ApiIdea;

fn render_idea_detail(f: &mut Frame, area: Rect, idea: &ApiIdea) {
    let author = idea.author.as_ref().map(|a| {
        a.display_name.clone().unwrap_or_else(|| a.username.clone())
    }).unwrap_or_else(|| "不明".to_string());

    let mut text = String::new();
    text.push_str(&format!("{}  {}\n", pad_width("ID", 8),   idea.id));
    text.push_str(&format!("{}  {}\n", pad_width("状態", 8), idea.status));
    text.push_str(&format!("{}  {}\n", pad_width("作者", 8), author));
    text.push_str("\n--- 内容 ---\n");
    text.push_str(&idea.content);

    let para = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL)
            .title(format!(" {} ", idea.title))
            .border_style(Style::default().fg(COLOR_ACCENT)))
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(COLOR_FG));
    f.render_widget(para, area);
}

pub fn render(app: &App, f: &mut Frame, header_area: Rect, body_area: Rect, footer_area: Rect, idx: usize) {
    let title = app.ideas.get(idx).map(|i| i.title.as_str()).unwrap_or("詳細");
    render_header(f, header_area, title, None);
    if let Some(idea) = app.ideas.get(idx) {
        render_idea_detail(f, body_area, idea);
    }
    render_footer(f, footer_area, &[("b", "戻る"), ("q", "終了")]);
}
