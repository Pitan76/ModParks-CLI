// src/api_models.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiProject {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "iconUrl")]
    pub icon_url: Option<String>,
    #[serde(rename = "type")]
    pub project_type: String,
    pub license: String,
    pub downloads: Downloads,
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    pub author: Option<Author>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(dead_code)]
pub struct ApiProjectDetail {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "iconUrl")]
    pub icon_url: Option<String>,
    #[serde(rename = "type")]
    pub project_type: String,
    pub license: String,
    pub downloads: Downloads,
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    pub author: Author,
    pub tags: Vec<String>,
    pub dependencies: Vec<Dependency>,
    pub dependents: Vec<Dependency>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Downloads {
    pub total: u32,
    pub native: u32,
    pub modrinth: Option<u32>,
    pub curseforge: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Author {
    pub username: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(dead_code)]
pub struct Dependency {
    pub id: String,
    #[serde(rename = "dependencyType")]
    pub dependency_type: String,
    pub project: DependencyProject,
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(dead_code)]
pub struct DependencyProject {
    pub id: String,
    pub slug: String,
    pub name: String,
    #[serde(rename = "iconUrl")]
    pub icon_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiIdea {
    pub id: String,
    pub title: String,
    pub content: String,
    pub status: String,
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    pub author: Option<Author>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiComment {
    pub id: String,
    pub content: String,
    pub author: Option<Author>,
    #[serde(rename = "createdAt")]
    pub created_at: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AuthMe {
    pub id: String,
    pub role: String,
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct CreateProjectReq {
    pub name: String,
    pub slug: String,
    pub description: String,
    #[serde(rename = "type")]
    pub project_type: String,
}

#[derive(Debug, Serialize)]
pub struct UpdateProjectReq {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub project_type: Option<String>,
}
