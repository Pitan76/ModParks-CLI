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
use crate::api_models::{ApiProject, ApiIdea};

pub const COLOR_ACCENT: Color = Color::Rgb(99, 179, 237);
pub const COLOR_DIM: Color    = Color::Rgb(120, 120, 140);
pub const COLOR_BG: Color     = Color::Rgb(15, 15, 25);
pub const COLOR_FG: Color     = Color::White;
pub const COLOR_SELECT: Color = Color::Rgb(44, 82, 130);

fn pad_width(s: &str, target: usize) -> String {
    let w = s.width();
    if w >= target { s.to_string() } else { format!("{}{}", s, " ".repeat(target - w)) }
}

fn rpad_width(s: &str, target: usize) -> String {
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

pub fn render_project_detail(f: &mut Frame, area: Rect, project: &ApiProject) {
    let author = project.author.as_ref().map(|a| {
        a.display_name.clone().unwrap_or_else(|| a.username.clone())
    }).unwrap_or_else(|| "不明".to_string());

    let tags = project.tags.as_deref().unwrap_or(&[]).join(", ");

    let rows: &[(&str, &str)] = &[
        ("名前",           &project.name),
        ("スラッグ",       &project.slug),
        ("作者",           &author),
        ("ライセンス",     &project.license),
    ];
    let mut text = String::new();
    for (label, value) in rows {
        text.push_str(&format!("{}  {}\n", pad_width(label, 8), value));
    }
    text.push_str(&format!("{}  {}\n", pad_width("DL数", 8), project.downloads.total));
    text.push_str(&format!("{}  {}\n", pad_width("タグ", 8),
        if tags.is_empty() { "なし".to_string() } else { tags }));
    text.push_str("\n--- 説明 ---\n");
    text.push_str(project.description.as_deref().unwrap_or("なし"));

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

pub fn render_idea_detail(f: &mut Frame, area: Rect, idea: &ApiIdea) {
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

pub fn render_help(f: &mut Frame, area: Rect) {
    let key_w = 14usize;
    let entries: &[(&str, &str)] = &[
        ("Up / Down",   "選択を移動"),
        ("Left / Right","ページ移動"),
        ("Enter",       "詳細を表示 / ログイン実行"),
        ("b / BS",      "前の画面に戻る"),
        ("p",           "プロジェクト一覧"),
        ("i",           "アイデア一覧"),
        ("r",           "再取得（キャッシュ無視）"),
        ("/",           "検索"),
        ("l",           "表示件数変更 (20/40/80)"),
        ("?",           "このヘルプを表示"),
        ("q / Esc",     "終了"),
    ];
    let mut lines = vec![
        Line::from(vec![Span::styled("キーバインド", Style::default().fg(COLOR_ACCENT).add_modifier(Modifier::BOLD))]),
        Line::from(""),
    ];
    for (key, desc) in entries {
        lines.push(Line::from(vec![
            Span::styled(format!("  {}", pad_width(key, key_w)), Style::default().fg(COLOR_ACCENT)),
            Span::raw(*desc),
        ]));
    }
    let para = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" ヘルプ ").border_style(Style::default().fg(COLOR_ACCENT)))
        .wrap(Wrap { trim: false });
    f.render_widget(para, area);
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

pub fn split_login(area: Rect) -> (Rect, Rect, Rect, Rect, Rect) {
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

pub fn split_project_form(area: Rect) -> (Rect, Rect, Rect, Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Name
            Constraint::Length(3), // Slug
            Constraint::Min(5),    // Description
            Constraint::Length(3), // Type
            Constraint::Length(3), // Message
        ])
        .split(area);
    (chunks[0], chunks[1], chunks[2], chunks[3], chunks[4])
}

pub fn render_profile(f: &mut Frame, area: Rect, user: Option<&crate::api_models::AuthMe>) {
    let mut text = String::new();
    if let Some(me) = user {
        text.push_str(&format!("{}  {}\n", pad_width("ユーザー名", 10), me.username));
        text.push_str(&format!("{}  {}\n", pad_width("権限", 10), me.role));
        text.push_str("\n--- 詳細なプロフィールはブラウザ等から確認できます ---\n");
    } else {
        text.push_str("未ログイン、またはユーザー情報が取得できません。");
    }

    let para = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL)
            .title(" プロフィール ")
            .border_style(Style::default().fg(COLOR_ACCENT)))
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(COLOR_FG));
    f.render_widget(para, area);
}

pub fn split_download_form(area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Path input
            Constraint::Min(0),    // Message
        ])
        .split(area);
    (chunks[0], chunks[1])
}

pub fn split_upload_form(area: Rect) -> (Rect, Rect, Rect, Rect, Rect, Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // file/url
            Constraint::Length(3), // file_name
            Constraint::Length(3), // version_number
            Constraint::Length(3), // loaders
            Constraint::Length(3), // mc_versions
            Constraint::Min(5),    // changelog
            Constraint::Length(3), // Message
        ])
        .split(area);
    (chunks[0], chunks[1], chunks[2], chunks[3], chunks[4], chunks[5], chunks[6])
}