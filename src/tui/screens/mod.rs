pub mod login;
pub mod project_list;
pub mod my_projects;
pub mod profile;
pub mod project_detail;
pub mod project_form;
pub mod idea_list;
pub mod idea_detail;
pub mod help;

use ratatui::{Frame, layout::Rect};
use crate::tui::App;

// Common trait for screens could be defined here in the future
// pub trait ScreenView {
//     fn render(&self, app: &mut App, f: &mut Frame, area: Rect);
// }
