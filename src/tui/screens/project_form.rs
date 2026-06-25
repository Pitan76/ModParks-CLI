use ratatui::{Frame, layout::Rect};
use crate::tui::{App, InputMode, Screen};
use crate::tui::ui::{render_header, render_footer, split_project_form, render_input, render_error, render_loading};

pub fn render(app: &App, f: &mut Frame, header_area: Rect, body_area: Rect, footer_area: Rect) {
    let title = if matches!(app.screen, Screen::ProjectCreate) { "プロジェクト作成" } else { "プロジェクト編集" };
    render_header(f, header_area, title, None);
    let (name_a, slug_a, desc_a, type_a, msg_a) = split_project_form(body_area);
    
    let focus = if let InputMode::ProjectForm(i) = app.input_mode { i } else { 99 };
    
    render_input(f, name_a, "Name", &app.project_form_inputs[0], false, focus == 0);
    render_input(f, slug_a, "Slug", &app.project_form_inputs[1], false, focus == 1);
    render_input(f, desc_a, "Description", &app.project_form_inputs[2], false, focus == 2);
    render_input(f, type_a, "Type (mod/plugin)", &app.project_form_inputs[3], false, focus == 3);

    if let Some(err) = &app.error_msg {
        render_error(f, msg_a, err);
    } else if app.loading {
        render_loading(f, msg_a);
    }
    render_footer(f, footer_area, &[("Tab", "項目移動"), ("Enter", "保存"), ("Esc", "キャンセル")]);
}
