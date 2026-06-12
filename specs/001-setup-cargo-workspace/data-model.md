# Data Model: Cargo Workspaceと基本ランナーのセットアップ

本機能はプロジェクトの基盤構築であるため、アプリケーションレベルのデータモデルは存在しない。代わりに、物理的なコンポーネント構造を定義する。

## コンポーネント構造

### 1. `core` (Library Crate)
- **役割**: ゲームロジックのコア。ランナー層から呼び出される。
- **境界**: `lib.rs` を通じて、入力データと描画バッファの受け渡し口を提供する（本フェーズでは空のライブラリとして定義）。

### 2. `runner-debian` (Binary Crate)
- **役割**: Debian実機環境でのエントリポイント。
- **依存関係**: `core` (local path dependency)

### 3. `runner-wslg` (Binary Crate)
- **役割**: WSLg環境でのエントリポイント。
- **依存関係**: `core` (local path dependency)

## バリデーションルール
- `core` から `runner-*` への依存は禁止。
- 全てのバイナリは `cargo run -p` で独立して実行可能であること。
