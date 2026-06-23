// src/commands/mod.rs
pub mod login;
pub mod project;
pub mod version;
pub mod idea;
pub mod comment;
pub mod sync;
pub mod update;
pub mod download;
pub mod profile;

pub use login::login;
pub use project::{list_projects, list_my_projects, get_project, create_project, update_project};
pub use version::{list_versions, create_version};
pub use download::download_version;
pub use idea::{list_ideas, get_idea};
pub use comment::{list_comments, post_comment};
pub use sync::sync_project;
pub use update::*;
pub use profile::display_profile;
