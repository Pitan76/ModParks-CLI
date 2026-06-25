use ratatui::{Frame, layout::Rect};
use crate::tui::App;
use crate::tui::ui::{render_header, render_footer, render_error, render_loading, render_idea_list};

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
