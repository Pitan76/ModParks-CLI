// src/args.rs
use clap::{Parser, Subcommand, ArgAction};

#[derive(Parser)]
#[command(
    name = "modparks",
    version,
    author = "Pitan76",
    about = "ModParks CLI",
    disable_version_flag = true,
)]
pub struct Cli {
    /// バージョンを表示
    #[arg(short = 'v', short_alias = 'V', long = "version", action = ArgAction::Version)]
    pub version: Option<bool>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// API キーを保存する
    Login { api_key: String },
    /// プロジェクト一覧を取得
    Projects {
        #[arg(default_value_t = 1)]
        page: u32,
        #[arg(long = "page", hide = true)]
        page_opt: Option<u32>,
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
        #[arg(long, default_value = "mod", help = "mod, plugin, resourcepack, datapack, shader, modpack")]
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
        #[arg(long, help = "mod, plugin, resourcepack, datapack, shader, modpack")]
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
        #[arg(long, conflicts_with = "url")]
        file: Option<String>,
        #[arg(long, conflicts_with = "file")]
        url: Option<String>,
        #[arg(long)]
        file_name: Option<String>,
        #[arg(long)]
        version_number: String,
        #[arg(long = "loader")]
        loaders: Vec<String>,
        #[arg(long = "mc-version")]
        mc_versions: Vec<String>,
        #[arg(long)]
        changelog: Option<String>,
    },
    /// プロジェクトのバージョン（ファイル）をダウンロードする
    Download {
        slug: String,
        #[arg(long)]
        version: Option<String>,
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
    /// プロフィールを表示する (省略時は自分のプロフィール)
    Profile {
        username: Option<String>,
    },
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
