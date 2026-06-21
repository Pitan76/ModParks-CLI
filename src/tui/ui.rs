// src/tui/ui.rs
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};
use crate::api_models::{ApiProject, ApiIdea};

pub const COLOR_ACCENT: Color = Color::Rgb(99, 179, 237);   // ModParks ブルー
pub const COLOR_DIM: Color    = Color::Rgb(120, 120, 140);
pub const COLOR_BG: Color     = Color::Rgb(15, 15, 25);
pub const COLOR_FG: Color     = Color::White;
pub const COLOR_SELECT: Color = Color::Rgb(44, 82, 130);

pub fn render_header(f: &mut Frame, area: Rect, title: &str) {
    let header = Paragraph::new(format!(" ModParks CLI  |  {}", title))
        .style(Style::default().fg(COLOR_ACCENT).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(COLOR_ACCENT)));
    f.render_widget(header, area);
}

pub fn render_footer(f: &mut Frame, area: Rect, hints: &[(&str, &str)]) {
    let spans: Vec<Span> = hints.iter().flat_map(|(key, desc)| {
        vec![
            Span::styled(format!(" {} ", key), Style::default().fg(COLOR_ACCENT).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{} ", desc), Style::default().fg(COLOR_DIM)),
        ]
    }).collect();
    let footer = Paragraph::new(Line::from(spans))
        .block(Block::default().borders(Borders::TOP).border_style(Style::default().fg(COLOR_DIM)));
    f.render_widget(footer, area);
}

pub fn render_project_list(
    f: &mut Frame,
    area: Rect,
    projects: &[ApiProject],
    state: &mut ListState,
) {
    let items: Vec<ListItem> = projects.iter().map(|p| {
        let dl = format!("{} DL", p.downloads.total);
        let line = Line::from(vec![
            Span::styled(format!("  {:<40}", p.name), Style::default().fg(COLOR_FG)),
            Span::styled(format!(" {:>10}", dl), Style::default().fg(COLOR_DIM)),
            Span::styled(format!("  {}", p.slug), Style::default().fg(COLOR_DIM)),
        ]);
        ListItem::new(line)
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" プロジェクト一覧 ").border_style(Style::default().fg(COLOR_ACCENT)))
        .highlight_style(Style::default().bg(COLOR_SELECT).fg(COLOR_FG).add_modifier(Modifier::BOLD))
        .highlight_symbol("▶ ");
    f.render_stateful_widget(list, area, state);
}

pub fn render_project_detail(f: &mut Frame, area: Rect, project: &ApiProject) {
    let author = project.author.as_ref().map(|a| {
        a.display_name.clone().unwrap_or_else(|| a.username.clone())
    }).unwrap_or_else(|| "不明".to_string());

    let tags = project.tags.as_deref().unwrap_or(&[]).join(", ");
    let text = format!(
        "名前:         {}\nスラッグ:     {}\n作者:         {}\nライセンス:   {}\nダウンロード数: {}\nタグ:         {}\n\n--- 説明 ---\n{}",
        project.name,
        project.slug,
        author,
        project.license,
        project.downloads.total,
        if tags.is_empty() { "なし".to_string() } else { tags },
        project.description.as_deref().unwrap_or("なし"),
    );

    let para = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL)
            .title(format!(" {} ", project.name))
            .border_style(Style::default().fg(COLOR_ACCENT)))
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(COLOR_FG));
    f.render_widget(para, area);
}

pub fn render_idea_list(
    f: &mut Frame,
    area: Rect,
    ideas: &[ApiIdea],
    state: &mut ListState,
) {
    let items: Vec<ListItem> = ideas.iter().map(|idea| {
        let status_color = match idea.status.as_str() {
            "open"        => Color::Green,
            "in_progress" => Color::Yellow,
            "fulfilled"   => Color::Rgb(120, 120, 140),
            _             => Color::White,
        };
        let status_label = match idea.status.as_str() {
            "open"        => "[ 募集中 ]",
            "in_progress" => "[ 対応中 ]",
            "fulfilled"   => "[ 完了   ]",
            _             => "[  ---   ]",
        };
        let line = Line::from(vec![
            Span::styled(format!("  {:<12}", status_label), Style::default().fg(status_color)),
            Span::styled(format!(" {}", idea.title), Style::default().fg(COLOR_FG)),
        ]);
        ListItem::new(line)
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" アイデア一覧 ").border_style(Style::default().fg(COLOR_ACCENT)))
        .highlight_style(Style::default().bg(COLOR_SELECT).fg(COLOR_FG).add_modifier(Modifier::BOLD))
        .highlight_symbol("▶ ");
    f.render_stateful_widget(list, area, state);
}

pub fn render_idea_detail(f: &mut Frame, area: Rect, idea: &ApiIdea) {
    let author = idea.author.as_ref().map(|a| {
        a.display_name.clone().unwrap_or_else(|| a.username.clone())
    }).unwrap_or_else(|| "不明".to_string());

    let text = format!(
        "ID:     {}\n状態:   {}\n作者:   {}\n\n--- 内容 ---\n{}",
        idea.id, idea.status, author, idea.content,
    );

    let para = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL)
            .title(format!(" {} ", idea.title))
            .border_style(Style::default().fg(COLOR_ACCENT)))
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(COLOR_FG));
    f.render_widget(para, area);
}

pub fn render_loading(f: &mut Frame, area: Rect) {
    let para = Paragraph::new("  読み込み中...")
        .style(Style::default().fg(COLOR_DIM))
        .alignment(Alignment::Center);
    f.render_widget(para, area);
}

pub fn render_help(f: &mut Frame, area: Rect) {
    let text = vec![
        Line::from(vec![Span::styled("キーバインド", Style::default().fg(COLOR_ACCENT).add_modifier(Modifier::BOLD))]),
        Line::from(""),
        Line::from(vec![Span::styled("  ↑ / ↓     ", Style::default().fg(COLOR_ACCENT)), Span::raw("選択を移動")]),
        Line::from(vec![Span::styled("  Enter      ", Style::default().fg(COLOR_ACCENT)), Span::raw("詳細を表示")]),
        Line::from(vec![Span::styled("  b / BS     ", Style::default().fg(COLOR_ACCENT)), Span::raw("前の画面に戻る")]),
        Line::from(vec![Span::styled("  p          ", Style::default().fg(COLOR_ACCENT)), Span::raw("プロジェクト一覧")]),
        Line::from(vec![Span::styled("  i          ", Style::default().fg(COLOR_ACCENT)), Span::raw("アイデア一覧")]),
        Line::from(vec![Span::styled("  r          ", Style::default().fg(COLOR_ACCENT)), Span::raw("再取得（キャッシュ無視）")]),
        Line::from(vec![Span::styled("  ?          ", Style::default().fg(COLOR_ACCENT)), Span::raw("このヘルプを表示")]),
        Line::from(vec![Span::styled("  q / Esc    ", Style::default().fg(COLOR_ACCENT)), Span::raw("終了")]),
    ];
    let para = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title(" ヘルプ ").border_style(Style::default().fg(COLOR_ACCENT)))
        .wrap(Wrap { trim: false });
    f.render_widget(para, area);
}

pub fn split_layout(area: Rect) -> (Rect, Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(area);
    (chunks[0], chunks[1], chunks[2])
}