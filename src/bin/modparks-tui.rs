use std::env;
use std::process::{Command, exit};

fn main() {
    // 自身の引数を取得
    let args: Vec<String> = env::args().collect();
    
    // modparks-cli tui に引数をそのまま渡す
    let mut cmd = Command::new("modparks-cli");
    cmd.arg("tui");
    
    // 最初の引数(実行ファイルパス)はスキップして、それ以降の引数があれば追加
    if args.len() > 1 {
        cmd.args(&args[1..]);
    }

    // コマンドを実行し、結果を待機
    let status = match cmd.status() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("modparks-cli の起動に失敗しました: {}", e);
            eprintln!("modparks-cli がPATHに通っていることを確認してください。");
            exit(1);
        }
    };

    // 元のコマンドの終了コードをそのまま返す
    exit(status.code().unwrap_or(1));
}
