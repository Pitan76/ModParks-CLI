use ratatui::{Frame, layout::Rect};
use crate::tui::{App, InputMode};
use crate::tui::ui::{render_header, render_footer, split_search, render_input, render_error, render_loading, render_project_list};

pub fn render(app: &mut App, f: &mut Frame, header_area: Rect, body_area: Rect, footer_area: Rect) {
    let page_str = format!("{}件表示 | {}ページ", app.limit, app.project_page);
    render_header(f, header_area, "プロジェクト", Some(&page_str));
    let (search_area, list_area) = split_search(body_area);
    render_input(f, search_area, "検索 (/ で入力)", &app.search_input, false, app.input_mode == InputMode::Search);
    
    if let Some(err) = &app.error_msg {
        render_error(f, list_area, err);
    } else if app.loading {
        render_loading(f, list_area);
    } else {
        render_project_list(f, list_area, &app.projects, &mut app.project_state);
    }
    render_footer(f, footer_area, &[
        ("Enter", "詳細"), ("< / >", "ページ遷移"), ("l", "件数変更"), ("i", "アイデア"), ("m", "自分のプロジェクト"), ("u", "プロフィール"), ("L", "ログイン/ログアウト"), ("/", "検索"), ("r", "再取得"), ("?", "ヘルプ"), ("q", "終了"),
    ]);
}
