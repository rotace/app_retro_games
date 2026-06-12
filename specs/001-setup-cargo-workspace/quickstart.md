# Quickstart Validation Guide: Cargo Workspace Setup

## 検証シナリオ

### シナリオ1: Workspace全体のビルド確認
- **コマンド**: `cargo build`
- **期待される結果**: 全てのクレート (`core`, `runner-debian`, `runner-wslg`) が正常にコンパイルされること。

### シナリオ2: Debianランナーの実行確認
- **コマンド**: `cargo run -p runner-debian`
- **期待される結果**: 標準出力に `Hello, World!` または `Hello from runner-debian!` が表示されること。

### シナリオ3: WSLgランナーの実行確認
- **コマンド**: `cargo run -p runner-wslg`
- **期待される結果**: 標準出力に `Hello, World!` または `Hello from runner-wslg!` が表示されること。

## セットアップ手順
1. プロジェクトルートで `cargo build` を実行し、依存関係が正しく解決されていることを確認する。
2. 各ランナーを個別に実行し、出力を確認する。
