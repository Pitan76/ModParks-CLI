# ModParks CLI

## 概要
`modparks` は ModParks の API と連携するコマンドラインツールです。プロジェクトの一覧取得、個別プロジェクトの取得、バージョンやアイデア、コメントの取得・投稿、プロジェクトの同期、そして API キーの保存（login）をサポートします。

## インストール
### Linux (apt)
```bash
# 鍵とリポジトリの登録
curl -fsSL https://pitan76.github.io/apt-repo/pitan76.gpg | sudo gpg --dearmor -o /usr/share/keyrings/pitan76-archive-keyring.gpg
echo "deb [signed-by=/usr/share/keyrings/pitan76-archive-keyring.gpg] https://pitan76.github.io/apt-repo/ stable main" | sudo tee /etc/apt/sources.list.d/pitan76.list

# インストール
sudo apt update && sudo apt install modparks-cli
```

## 前提条件
- **Rust** (stable) がインストールされていること
- `cargo` がパスに通っていること

## ビルド手順
```bash
# 依存パッケージを取得し、リリースビルドを作成
cargo build --release
```
ビルドが成功すると、実行バイナリは `target/release/modparks-cli.exe` に生成されます。

## インストール（オプション）
ビルドしたバイナリをパスの通ったディレクトリにコピーすれば、どこからでも `modparks-cli` コマンドを呼び出せます。
```bash
# 例: Windows のユーザープロファイルの bin ディレクトリへコピー
copy target\release\modparks-cli.exe %USERPROFILE%\bin\
```



## 使い方
```bash
modparks-cli --help
```
以下は主なサブコマンドです。

| コマンド | 説明 |
|---|---|
| `login <API_KEY>` | API キーを保存し、以降のリクエストで認証に使用します |
| `logout` | 保存されたAPIキーを削除しログアウトします |
| `profile [username]` | ユーザー情報を表示します (省略時は自身のプロフィール) |
| `projects [PAGE] [--limit <n>]` | プロジェクト一覧を取得（デフォルト 20 件） |
| `my-projects [PAGE] [--limit <n>]`| 自分が作成したプロジェクト一覧を取得 |
| `project <slug>` | 指定スラッグのプロジェクト詳細を取得 |
| `project-create --name <NAME> ...` | プロジェクトを新規作成します |
| `project-edit <slug> ...` | プロジェクトの情報を編集します |
| `versions <slug> [--limit <n>]` | プロジェクトのバージョン一覧を取得 |
| `version-create <slug> ...` | プロジェクトに新しいバージョンを登録（ファイルやURLをアップロード）します |
| `download <slug> [--version <ver>]` | プロジェクトのバージョン（ファイル）をダウンロードします |
| `ideas [--limit <n>]` | アイデア一覧を取得 |
| `idea <id>` | 指定IDのアイデア詳細を取得 |
| `comments <slug>` | プロジェクトのコメント一覧を取得 |
| `comment-post <slug> <content>`| プロジェクトにコメントを投稿 |
| `sync <slug>` | プロジェクトを外部プラットフォームと同期 |
| `tui` | ターミナルUI (TUI) を起動してインタラクティブに操作します |
| `update` | modparks-cli 自身を最新バージョンへアップデートします |
| `cache-clear` | キャッシュされたレスポンスを削除します |
| `help` | 各コマンドのヘルプメッセージを表示します |

## TUI (Terminal UI)
`modparks-cli tui` コマンドで起動できる TUI モードでは、方向キーやショートカットでプロジェクトやアイデアを簡単に閲覧できます。

## 設定ファイル
設定は JSON 形式で保存され、デフォルトの場所は以下です。
- Windows: `%LOCALAPPDATA%/modparks-cli/config.json`
- Linux/macOS: `$HOME/.config/modparks-cli/config.json`

設定ファイルの例:
```json
{
  "api_base_url": "https://modparks.pitan76.net/api/v1",
  "api_key": "YOUR_API_KEY",
  "cache_enabled": true,
  "cache_ttl_seconds": 300
}
```
`login` コマンドで API キーを保存すると自動的に上記ファイルが作成・更新されます。

## リリース（公開）手順
このリポジトリでは GitHub Actions を利用して、タグが Push されたときに自動でビルド＆ GitHub Releases への公開を行います。

リリースを行うには以下の手順を実行します。

1. `Cargo.toml` の `version` を更新し、コミットします。
2. Git でアノテーション付きタグを作成します。
3. タグをリモートに Push します。

```bash
# 例: v0.1.0 をリリースする場合
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```

## タスク
- 右クリックで開けるようにする
- modparks-tuiでtuiを直接開けるようにする


その後、GitHub Actions 側で自動的に **Windows**, **macOS**, **Linux** 向けの実行ファイルがビルドされ、Releases にアップロードされます。

## ライセンス
この CLI は MIT ライセンスの下で配布されています。詳細はリポジトリの `LICENSE` ファイルをご覧ください。
