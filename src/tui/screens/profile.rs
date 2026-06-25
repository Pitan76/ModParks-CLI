use ratatui::{
    Frame, layout::Rect,
    style::Style,
    widgets::{Block, Borders, Paragraph, Wrap},
};
use crate::tui::App;
use crate::tui::ui::{render_header, render_footer, pad_width, COLOR_ACCENT, COLOR_FG};
use crate::api_models::{Author, AuthMe, ApiProject};

fn render_profile(f: &mut Frame, area: Rect, author_opt: Option<&Author>, me: Option<&AuthMe>, my_projects: &[ApiProject]) {
    let mut text = String::new();
    if let Some(author) = author_opt {
        text.push_str(&format!("{}  {}\n", pad_width("ユーザー名", 10), author.username));
        if let Some(dname) = &author.display_name {
            text.push_str(&format!("{}  {}\n", pad_width("表示名", 10), dname));
        }
        text.push_str("\n--- 詳細なプロフィールはブラウザ等から確認できます ---\n");
    } else if let Some(user) = me {
        text.push_str(&format!("{}  {}\n", pad_width("ユーザー名", 10), user.username));
        // ロールは表示しません
        let project_count = my_projects.len();
        let total_downloads: u32 = my_projects.iter().map(|p| p.downloads.total).sum();
        text.push_str(&format!("{}  {}\n", pad_width("プロジェクト数", 10), project_count));
        text.push_str(&format!("{}  {}\n", pad_width("総ダウンロード数", 10), total_downloads));
        if !my_projects.is_empty() {
            text.push_str("\n作品一覧:\n");
            for proj in my_projects {
                text.push_str(&format!(" - {}\n", proj.name));
            }
        }
        text.push_str("\n--- 詳細なプロフィールはブラウザ等から確認できます ---\n");
    } else {
        text.push_str("未ログイン、またはユーザー情報が取得できません。\n");
    }

    let para = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL)
            .title(" プロフィール ")
            .border_style(Style::default().fg(COLOR_ACCENT)))
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(COLOR_FG));
    f.render_widget(para, area);
}

pub fn render(app: &App, f: &mut Frame, header_area: Rect, body_area: Rect, footer_area: Rect, author_opt: Option<&Author>) {
    render_header(f, header_area, "プロフィール", None);
    render_profile(f, body_area, author_opt, app.current_user.as_ref(), &app.my_projects);
    render_footer(f, footer_area, &[("b", "戻る"), ("q", "終了")]);
}
