// src/api_models.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct ApiProject {
    id: String,
    slug: String,
    name: String,
    description: Option<String>,
    #[serde(rename = "iconUrl")]
    icon_url: Option<String>,
    #[serde(rename = "type")]
    project_type: String,
    license: String,
    downloads: Downloads,
    #[serde(rename = "createdAt")]
    created_at: i64,
    #[serde(rename = "updatedAt")]
    updated_at: i64,
    author: Option<Author>,
    categories: Option<Vec<String>>, // Adjusted from original Option<Vec<Vec<String>>> typo
    tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ApiProjectDetail {
    id: String,
    slug: String,
    name: String,
    description: Option<String>,
    #[serde(rename = "iconUrl")]
    icon_url: Option<String>,
    #[serde(rename = "type")]
    project_type: String,
    license: String,
    downloads: Downloads,
    #[serde(rename = "createdAt")]
    created_at: i64,
    #[serde(rename = "updatedAt")]
    updated_at: i64,
    author: Author,
    tags: Vec<String>,
    dependencies: Vec<Dependency>,
    dependents: Vec<Dependency>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Downloads {
    total: u32,
    native: u32,
    #[serde(flatten)]
    extra: std::collections::HashMap<String, u32>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Author {
    username: String,
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    #[serde(rename = "avatarUrl")]
    avatar_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Dependency {
    id: String,
    #[serde(rename = "dependencyType")]
    dependency_type: String,
    project: DependencyProject,
}

#[derive(Debug, Deserialize, Serialize)]
struct DependencyProject {
    id: String,
    slug: String,
    name: String,
    #[serde(rename = "iconUrl")]
    icon_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ApiIdea {
    id: String,
    title: String,
    content: String,
    status: String,
    #[serde(rename = "createdAt")]
    created_at: i64,
    #[serde(rename = "updatedAt")]
    updated_at: i64,
    author: Option<Author>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ApiComment {
    id: String,
    content: String,
    author: Option<Author>,
    #[serde(rename = "createdAt")]
    created_at: i64,
}

// Paginated response and meta structs remain in commands.rs for reuse
