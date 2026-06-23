use anyhow::{Result, anyhow};
use crate::config::Config;
use crate::api_client::{cached_get, create_project as api_create_project, update_project as api_update_project};
use crate::api_models::{ApiProject, CreateProjectReq, UpdateProjectReq};
use crate::pagination::PaginatedResponse;

pub async fn list_projects(limit: u32, offset: u32) -> Result<()> {
    let cfg = Config::load()?;
    let url = format!("{}/projects?limit={}&offset={}", cfg.api_base_url, limit, offset);
    let paginated: PaginatedResponse<ApiProject> = cached_get(&url, &cfg).await?;
    for p in &paginated.data {
        println!("- {} ({})", p.name, p.slug);
    }
    println!("取得件数: {} (limit={}, offset={})", paginated.meta.count, paginated.meta.limit, paginated.meta.offset);
    Ok(())
}

pub async fn list_my_projects(limit: u32, offset: u32) -> Result<()> {
    let cfg = Config::load()?;
    if cfg.api_key.is_empty() {
        return Err(anyhow!("ログインしていません。まず login コマンドを実行してください。"));
    }
    let me = crate::api_client::auth_me(&cfg).await?;
    let url = format!("{}/projects?limit={}&offset={}&author={}", cfg.api_base_url, limit, offset, me.username);
    let paginated: PaginatedResponse<ApiProject> = cached_get(&url, &cfg).await?;
    println!("==== {} のプロジェクト ====", me.username);
    for p in &paginated.data {
        println!("- {} ({})", p.name, p.slug);
    }
    println!("取得件数: {} (limit={}, offset={})", paginated.meta.count, paginated.meta.limit, paginated.meta.offset);
    Ok(())
}


pub async fn get_project(slug: &str) -> Result<()> {
    let cfg = Config::load()?;
    let url = format!("{}/projects/{}", cfg.api_base_url, slug);

    #[derive(serde::Deserialize, serde::Serialize)]
    struct Wrapper { data: ApiProject }

    let wrapper: Wrapper = cached_get(&url, &cfg).await
        .or_else(|_| {
            // data ラップなしでも試みる
            Err(anyhow!("プロジェクト取得に失敗しました"))
        })?;
    let project = wrapper.data;
    println!("名前:     {} ({})", project.name, project.slug);
    println!("説明:     {}", project.description.unwrap_or_default());
    println!("ダウンロード数: {}", project.downloads.total);
    if let Some(tags) = &project.tags {
        if !tags.is_empty() {
            println!("タグ:     {}", tags.join(", "));
        }
    }
    Ok(())
}

pub async fn create_project(name: String, slug: String, description: String, project_type: String) -> Result<()> {
    let cfg = Config::load()?;
    let req = CreateProjectReq {
        name,
        slug,
        description,
        project_type,
    };
    
    api_create_project(&cfg, &req).await?;
    println!("プロジェクトを作成しました！");
    Ok(())
}

pub async fn update_project(slug: String, name: Option<String>, new_slug: Option<String>, description: Option<String>, project_type: Option<String>) -> Result<()> {
    let cfg = Config::load()?;
    let req = UpdateProjectReq {
        name,
        slug: new_slug,
        description,
        project_type,
    };
    
    api_update_project(&cfg, &slug, &req).await?;
    println!("プロジェクトを更新しました！");
    Ok(())
}
