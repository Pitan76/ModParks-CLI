// src/tui/ui.rs
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};
use unicode_width::UnicodeWidthStr;
use tui_input::Input;
use crate::api_models::ApiProject;

pub const COLOR_ACCENT: Color = Color::Rgb(99, 179, 237);
pub const COLOR_DIM: Color    = Color::Rgb(120, 120, 140);
pub const COLOR_BG: Color     = Color::Rgb(15, 15, 25);
pub const COLOR_FG: Color     = Color::White;
pub const COLOR_SELECT: Color = Color::Rgb(44, 82, 130);
pub const COLOR_ERROR: Color  = Color::Red;

pub fn pad_width(s: &str, target: usize) -> String {
    let w = s.width();
    if w >= target { s.to_string() } else { format!("{}{}", s, " ".repeat(target - w)) }
}

pub fn rpad_width(s: &str, target: usize) -> String {
    let w = s.width();
    if w >= target { s.to_string() } else { format!("{}{}", " ".repeat(target - w), s) }
}

pub fn render_header(f: &mut Frame, area: Rect, title: &str, page_info: Option<&str>) {
    let title_text = if let Some(info) = page_info {
        format!(" ModParks CLI  |  {} ({})", title, info)
    } else {
        format!(" ModParks CLI  |  {}", title)
    };
    
    let header = Paragraph::new(title_text)
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
            Span::styled(format!("  {}", pad_width(&p.name, 40)), Style::default().fg(COLOR_FG)),
            Span::styled(format!(" {}", rpad_width(&dl, 10)),     Style::default().fg(COLOR_DIM)),
            Span::styled(format!("  {}", p.slug),                  Style::default().fg(COLOR_DIM)),
        ]);
        ListItem::new(line)
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL)
            .title(" プロジェクト一覧 ")
            .border_style(Style::default().fg(COLOR_ACCENT)))
        .highlight_style(Style::default().bg(COLOR_SELECT).fg(COLOR_FG).add_modifier(Modifier::BOLD))
        .highlight_symbol("▶ ");
    f.render_stateful_widget(list, area, state);
}



pub fn render_loading(f: &mut Frame, area: Rect) {
    let para = Paragraph::new("  読み込み中...")
        .style(Style::default().fg(COLOR_DIM))
        .alignment(Alignment::Center);
    f.render_widget(para, area);
}

pub fn render_error(f: &mut Frame, area: Rect, message: &str) {
    let text = format!("エラー: {}\n\n(Enter, Esc, q 等で閉じる)", message);
    let p = Paragraph::new(text)
        .style(Style::default().fg(Color::Red).bg(COLOR_BG))
        .block(Block::default().borders(Borders::ALL).title(" Error ").style(Style::default().fg(Color::Red)));
    f.render_widget(p, area);
}



pub fn render_input(f: &mut Frame, area: Rect, title: &str, input: &Input, is_password: bool, is_active: bool) {
    let text = if is_password {
        "*".repeat(input.value().chars().count())
    } else {
        input.value().to_string()
    };
    
    let border_style = if is_active {
        Style::default().fg(COLOR_FG).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(COLOR_DIM)
    };

    let para = Paragraph::new(text)
        .style(Style::default().fg(COLOR_FG))
        .block(Block::default().borders(Borders::ALL).title(format!(" {} ", title)).border_style(border_style))
        .wrap(Wrap { trim: false });
    f.render_widget(para, area);
    
    // カーソル表示
    if is_active {
        let width = area.width.saturating_sub(2).max(1);
        let cx = (input.visual_cursor() as u16) % width;
        let cy = (input.visual_cursor() as u16) / width;
        
        let cx_actual = (area.x + 1 + cx).min(area.right().saturating_sub(2));
        let cy_actual = (area.y + 1 + cy).min(area.bottom().saturating_sub(2));
        f.set_cursor_position((cx_actual, cy_actual));
    }
}

pub fn split_layout(area: Rect) -> (Rect, Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Header
            Constraint::Min(0),    // Body
            Constraint::Length(2), // Footer
        ])
        .split(area);
    (chunks[0], chunks[1], chunks[2])
}

pub fn split_search(area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search bar
            Constraint::Min(0),    // List
        ])
        .split(area);
    (chunks[0], chunks[1])
}
