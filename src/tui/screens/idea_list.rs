use ratatui::{
    Frame, layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
};
use crate::tui::App;
use crate::tui::ui::{render_header, render_footer, render_error, render_loading, COLOR_ACCENT, COLOR_FG, COLOR_SELECT};
use crate::api_models::ApiIdea;

fn render_idea_list(
    f: &mut Frame,
    area: Rect,
    ideas: &[ApiIdea],
    state: &mut ListState,
) {
    let items: Vec<ListItem> = ideas.iter().map(|idea| {
        let (status_color, status_label) = match idea.status.as_str() {
            "open"        => (Color::Green,              "[open      ]"),
            "in_progress" => (Color::Yellow,             "[in_progress]"),
            "fulfilled"   => (Color::Rgb(120, 120, 140), "[fulfilled ]"),
            s             => (Color::White,              s),
        };
        let line = Line::from(vec![
            Span::styled(format!("  {}", status_label),              Style::default().fg(status_color)),
            Span::styled(format!("  {}", idea.title), Style::default().fg(COLOR_FG)),
        ]);
        ListItem::new(line)
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL)
            .title(" アイデア一覧 ")
            .border_style(Style::default().fg(COLOR_ACCENT)))
        .highlight_style(Style::default().bg(COLOR_SELECT).fg(COLOR_FG).add_modifier(Modifier::BOLD))
        .highlight_symbol("▶ ");
    f.render_stateful_widget(list, area, state);
}

pub fn render(app: &mut App, f: &mut Frame, header_area: Rect, body_area: Rect, footer_area: Rect) {
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
        ("Enter", "詳細"), ("< / >", "ページ遷移"), ("p", "プロジェクト"), ("L", "ログイン/ログアウト"), ("r", "再取得"), ("?", "ヘルプ"), ("q", "終了"),
    ]);
}
