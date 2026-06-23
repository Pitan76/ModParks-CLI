use clap::{Parser, Subcommand, ArgAction, CommandFactory};
mod api_client;
mod api_models;
mod cache;
mod commands;
mod config;
mod pagination;
mod tui;

#[derive(Parser)]
#[command(
    name = "modparks",
    version,
    author = "Pitan76",
    about = "ModParks CLI",
    // -v と -V の両方でバージョンを表示
    disable_version_flag = true,
)]
struct Cli {
    /// バージョンを表示
    #[arg(short = 'v', short_alias = 'V', long = "version", action = ArgAction::Version)]
    version: Option<bool>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// API キーを保存する
    Login { api_key: String },

    /// プロジェクト一覧を取得
    ///
    /// PAGE を指定するとそのページを取得します（1始まり）。
    /// 例: modparks-cli projects 2
    Projects {
        /// ページ番号（1始まり）。例: projects 2
        #[arg(default_value_t = 1)]
        page: u32,
        /// --page オプション（位置引数と同じ）
        #[arg(long = "page", hide = true)]
        page_opt: Option<u32>,
        /// 1ページあたりの件数
        #[arg(short, long, default_value_t = 20)]
        limit: u32,
    },

    /// 指定スラッグのプロジェクト詳細を取得
    Project { slug: String },

    /// プロジェクトを新規作成
    ProjectCreate {
        #[arg(long)]
        name: String,
        #[arg(long)]
        slug: String,
        #[arg(long, default_value_t = String::new())]
        description: String,
        #[arg(long, default_value = "mod")]
        project_type: String,
    },

    /// プロジェクト情報を編集
    ProjectEdit {
        slug: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        new_slug: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        project_type: Option<String>,
    },

    /// プロジェクトのバージョン一覧を取得
    Versions {
        slug: String,
        #[arg(short, long, default_value_t = 20)]
        limit: u32,
    },

    /// プロジェクトのバージョンを新規登録する
    VersionCreate {
        slug: String,
        /// アップロードするローカルファイルのパス (.jar, .zip など)
        #[arg(long, conflicts_with = "url")]
        file: Option<String>,
        /// 外部ダウンロードURL
        #[arg(long, conflicts_with = "file")]
        url: Option<String>,
        /// 外部ダウンロード時のファイル名 (URL指定時に推奨、未指定時はURLから抽出)
        #[arg(long)]
        file_name: Option<String>,
        /// バージョン番号
        #[arg(long)]
        version_number: String,
        /// 対応ローダー (例: --loader fabric --loader forge)
        #[arg(long = "loader")]
        loaders: Vec<String>,
        /// 対応Minecraftバージョン (例: --mc-version 1.20.1)
        #[arg(long = "mc-version")]
        mc_versions: Vec<String>,
        /// 変更ログ
        #[arg(long)]
        changelog: Option<String>,
    },

    /// プロジェクトのバージョン（ファイル）をダウンロードする
    Download {
        /// プロジェクトのスラグ
        slug: String,
        /// ダウンロードするバージョン番号 (未指定時は最新版)
        #[arg(long)]
        version: Option<String>,
        /// 出力先ディレクトリまたはファイル名 (未指定時はカレントディレクトリに保存)
        #[arg(long)]
        out: Option<String>,
    },

    /// アイデア一覧を取得
    Ideas {
        #[arg(short, long, default_value_t = 20)]
        limit: u32,
    },

    /// 指定 ID のアイデア詳細を取得
    Idea { id: String },

    /// プロジェクトのコメント一覧を取得
    Comments { slug: String },

    /// プロジェクトにコメントを投稿
    CommentPost { slug: String, content: String },

    /// プロジェクトを外部プラットフォームと同期
    Sync { slug: String },

    /// TUI を起動する
    Tui,

    /// 最新バージョンへアップデートする
    Update,

    /// ログアウトする (API キーを削除)
    Logout,

    /// 自分のプロフィールを表示する
    Profile,

    /// 自分の作成したプロジェクトの一覧を取得する
    MyProjects {
        #[arg(default_value_t = 1)]
        page: u32,
        #[arg(short, long, default_value_t = 20)]
        limit: u32,
    },

    /// キャッシュをクリアする
    CacheClear,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    if let Some(cmd) = cli.command {
        match cmd {
            Commands::Login { api_key } => commands::login(&api_key).await?,
            Commands::Update => commands::update()?,
            Commands::Logout => {
                let mut cfg = config::Config::load()?;
                cfg.api_key = String::new();
                cfg.save()?;
                println!("ログアウトしました (API キーを削除しました)。");
            }
            Commands::Profile => commands::display_profile().await?,
            Commands::MyProjects { page, limit } => {
                let offset = page.saturating_sub(1) * limit;
                commands::list_my_projects(limit, offset).await?
            }
            Commands::Projects { page, page_opt, limit } => {
                let p = page_opt.unwrap_or(page);
                let offset = p.saturating_sub(1) * limit;
                commands::list_projects(limit, offset).await?
            }
            Commands::Project { slug } => commands::get_project(&slug).await?,
            Commands::ProjectCreate { name, slug, description, project_type } => {
                commands::create_project(name, slug, description, project_type).await?
            }
            Commands::ProjectEdit { slug, name, new_slug, description, project_type } => {
                commands::update_project(slug, name, new_slug, description, project_type).await?
            }
            Commands::Versions { slug, limit } => commands::list_versions(&slug, limit).await?,
            Commands::VersionCreate { slug, file, url, file_name, version_number, loaders, mc_versions, changelog } => {
                commands::create_version(slug, file, url, file_name, version_number, loaders, mc_versions, changelog).await?
            }
            Commands::Download { slug, version, out } => commands::download_version(slug, version, out).await?,
            Commands::Ideas { limit } => commands::list_ideas(limit).await?,
            Commands::Idea { id } => commands::get_idea(&id).await?,
            Commands::Comments { slug } => commands::list_comments(&slug).await?,
            Commands::CommentPost { slug, content } => commands::post_comment(&slug, &content).await?,
            Commands::Sync { slug } => commands::sync_project(&slug).await?,
            Commands::Tui => tui::run_tui().await?,
            Commands::CacheClear => {
                let n = cache::clear()?;
                println!("{}件のキャッシュを削除しました。", n);
            }
        }
    } else {
        // 実行ファイル名を確認
        let exe_name = std::env::args().next().unwrap_or_default().to_lowercase();
        if exe_name.contains("modparks-tui") {
            // エイリアス (modparks-tui) 経由で呼ばれた場合のみTUIを起動
            tui::run_tui().await?;
        } else {
            // modparks-cli で引数なしの場合はヘルプを表示
            Cli::command().print_help()?;
        }
    }
    Ok(())
}