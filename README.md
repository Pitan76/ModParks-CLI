# ModParks CLI

## 概要
`modparks` は ModParks の API と連携するコマンドラインツールです。プロジェクトの一覧取得、個別プロジェクトの取得、バージョンやアイデア、コメントの取得・投稿、プロジェクトの同期、そして API キーの保存（login）をサポートします。

## 前提条件
- **Rust** (stable) がインストールされていること
- `cargo` がパスに通っていること
- インターネットに接続できる環境

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
modparks --help
```
以下は主なサブコマンドです。

| コマンド | 説明 |
|---|---|
| `login <API_KEY>` | API キーを保存し、以降のリクエストで認証に使用します |
| `projects [--limit <n>] [--offset <n>]` | プロジェクト一覧を取得（デフォルト 20 件、最大 80 件） |
| `project <slug>` | 指定スラッグのプロジェクト詳細を取得 |
| `versions <slug> [--limit <n>]` | プロジェクトのバージョン一覧を取得 |
| `ideas [--limit <n>]` | アイデア一覧を取得 |
| `comments <slug>` | プロジェクトのコメント一覧を取得 |
| `comment post <slug> <content>` | コメントを投稿 |
| `sync <slug>` | プロジェクトを外部プラットフォームと同期 |

## 設定ファイル
設定は JSON 形式で保存され、デフォルトの場所は以下です。
- Windows: `%APPDATA%/modparks-cli/config.json`
- Linux/macOS: `$HOME/.config/modparks-cli/config.json`

設定ファイルの例:
```json
{
  "api_base_url": "https://modparks.pitan76.net/api/v1",
  "api_key": "YOUR_API_KEY"
}
```
`login` コマンドで API キーを保存すると自動的に上記ファイルが作成・更新されます。

## ライセンス
この CLI は MIT ライセンスの下で配布されています。詳細はリポジトリの `LICENSE` ファイルをご覧ください。
