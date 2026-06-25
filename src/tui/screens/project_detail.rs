use ratatui::{Frame, layout::Rect};
use crate::tui::{App, InputMode, Screen};
use crate::tui::ui::{render_header, render_footer, split_download_form, split_upload_form, render_input, render_error, render_loading, render_project_detail};

pub fn render(app: &App, f: &mut Frame, header_area: Rect, body_area: Rect, footer_area: Rect, idx: usize) {
    let (title, project) = if matches!(app.screen, Screen::ProjectDetail(_)) {
        (app.projects.get(idx).map(|p| p.name.as_str()).unwrap_or("詳細"), app.projects.get(idx))
    } else {
        (app.my_projects.get(idx).map(|p| p.name.as_str()).unwrap_or("詳細"), app.my_projects.get(idx))
    };
    render_header(f, header_area, title, None);

    match app.input_mode {
        InputMode::DownloadForm => {
            let (path_a, msg_a) = split_download_form(body_area);
            render_input(f, path_a, "保存先ディレクトリパス", &app.download_input, false, true);
            if let Some(err) = &app.error_msg { render_error(f, msg_a, err); }
            else if app.loading { render_loading(f, msg_a); }
            render_footer(f, footer_area, &[("Enter", "ダウンロード実行"), ("Esc", "キャンセル")]);
        }
        InputMode::UploadForm(focus) => {
            let (file_a, name_a, ver_a, load_a, mc_a, change_a, msg_a) = split_upload_form(body_area);
            render_input(f, file_a, "File Path or URL", &app.upload_inputs[0], false, focus == 0);
            render_input(f, name_a, "File Name (optional for local file)", &app.upload_inputs[1], false, focus == 1);
            render_input(f, ver_a, "Version Number", &app.upload_inputs[2], false, focus == 2);
            render_input(f, load_a, "Loaders (comma separated)", &app.upload_inputs[3], false, focus == 3);
            render_input(f, mc_a, "Minecraft Versions (comma separated)", &app.upload_inputs[4], false, focus == 4);
            render_input(f, change_a, "Changelog", &app.upload_inputs[5], false, focus == 5);
            if let Some(err) = &app.error_msg { render_error(f, msg_a, err); }
            else if app.loading { render_loading(f, msg_a); }
            render_footer(f, footer_area, &[("Tab", "項目移動"), ("Enter", "アップロード実行"), ("Esc", "キャンセル")]);
        }
        _ => {
            if let Some(p) = project {
                render_project_detail(f, body_area, p);
            }
            if let Some(err) = &app.error_msg {
                let err_area = ratatui::layout::Rect::new(body_area.x + 5, body_area.y + 5, body_area.width.saturating_sub(10), 3);
                f.render_widget(ratatui::widgets::Clear, err_area);
                render_error(f, err_area, err);
            } else if app.loading {
                let load_area = ratatui::layout::Rect::new(body_area.x + 5, body_area.y + 5, body_area.width.saturating_sub(10), 3);
                f.render_widget(ratatui::widgets::Clear, load_area);
                render_loading(f, load_area);
            }
            render_footer(f, footer_area, &[("b", "戻る"), ("e", "編集"), ("d", "DL"), ("v", "UP"), ("u", "作者プロフ"), ("q", "終了")]);
        }
    }
}
