use ratatui::{
    Frame, layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use crate::tui::App;
use crate::tui::ui::{render_header, render_footer, pad_width, COLOR_ACCENT};

fn render_help(f: &mut Frame, area: Rect) {
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

pub fn render(_app: &App, f: &mut Frame, header_area: Rect, body_area: Rect, footer_area: Rect) {
    render_header(f, header_area, "ヘルプ", None);
    render_help(f, body_area);
    render_footer(f, footer_area, &[("b / Esc", "戻る"), ("q", "終了")]);
}
