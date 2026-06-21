// src/commands/update.rs
use anyhow::Result;
use self_update::cargo_crate_version;

pub fn update() -> Result<()> {
    println!("最新バージョンを確認しています...");

    let target = if cfg!(target_os = "windows") {
        "windows-x86_64.zip"
    } else if cfg!(target_os = "macos") {
        "macos-x86_64.tar.gz"
    } else {
        "linux-x86_64.tar.gz"
    };

    let status = self_update::backends::github::Update::configure()
        .repo_owner("Pitan76")
        .repo_name("ModParks-CLI")
        .bin_name("modparks-cli")
        .target(target)
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .build()?
        .update()?;

    if status.updated() {
        println!("アップデートが完了しました！バージョン: {}", status.version());
    } else {
        println!("最新のバージョン ({}) を使用中です。", status.version());
    }

    Ok(())
}