// src/commands/mod.rs
pub mod login;
pub mod project;
pub mod version;
pub mod idea;
pub mod comment;
pub mod sync;

pub use login::login;
pub use project::{list_projects, get_project};
pub use version::list_versions;
pub use idea::{list_ideas, get_idea};
pub use comment::{list_comments, post_comment};
pub use sync::sync_project;
