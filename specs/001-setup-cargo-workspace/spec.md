# Feature: Cargo Workspaceと基本ランナーのセットアップ

**User Story**: 開発者として、ADR 0002に準拠したRust Cargo Workspace構造を確立し、`runner-debian`および`runner-wslg`クレートを実行した際に「HelloWorld」メッセージが表示されることを確認したい。これにより、プロジェクトの機能的なベースラインを構築する。

---

## 1. 目標

- プロジェクトルートに、`doc/adr/0002-workspace-structure.md`で定義された構造に従ったCargo Workspaceを確立する。
- `runner-debian`と`runner-wslg`の2つの初期Rustクレートを作成する。
- これらの各クレート内に基本的な「Hello, World!」機能を実装し、実行して期待される出力を表示できるようにする。
- Workspaceが正しく設定され、ビルド可能であることを確認する。

## 2. ユーザーストーリー

- **ストーリー1**: 開発者がプロジェクトを初期化し、ADR 0002に従ってCargo Workspace構造がすでに設定されていることを確認する。
- **ストーリー2**: 開発者が`runner-debian`クレートに移動し、それをビルド・実行して、コンソールに「Hello, World!」が表示されることを確認する。
- **ストーリー3**: 開発者が`runner-wslg`クレートに移動し、それをビルド・実行して、コンソールに「Hello, World!」が表示されることを確認する。

## 3. 機能要件

- **FR1**: プロジェクトルートの`Cargo.toml`ファイルに、Cargo Workspaceを定義するものが含まれていること。
- **FR2**: ルートの`Cargo.toml`が、`runner-debian`および`runner-wslg`を含むWorkspaceメンバーを参照していること。
- **FR3**: `doc/adr/0002-workspace-structure.md`に準拠したディレクトリ構造が存在すること。CratesはWorkspaceルートの直下に配置される（例：`runner-debian/`）。
- **FR4**: `runner-debian`という名前のクレートがWorkspace構造内に存在すること。
- **FR5**: `runner-debian`クレートに`src/main.rs`ファイルが含まれていること。
- **FR6**: `runner-debian`クレートを実行すると（例：`cargo run -p runner-debian`）、標準出力に「Hello, World!」または「Hello from runner-debian!」という文字列が出力されること。
- **FR7**: `runner-wslg`という名前のクレートがWorkspace構造内に存在すること。
- **FR8**: `runner-wslg`クレートに`src/main.rs`ファイルが含まれていること。
- **FR9**: `runner-wslg`クレートを実行すると（例：`cargo run -p runner-wslg`）、標準出力に「Hello, World!」または「Hello from runner-wslg!」という文字列が出力されること。

## 4. 主要エンティティ

- この基本的なセットアップタスクには該当しません。

## 5. 前提条件

- `doc/adr/0002-workspace-structure.md`ファイルがプロジェクトルートに存在し、期待されるWorkspaceレイアウトを正確に記述していること。
- ADRが、ルート`Cargo.toml`の`members`配列と各クレートのディレクトリを含む、標準的なCargo Workspaceセットアップを指定していること。
- デフォルトのRustツールチェーンが利用可能で機能すること。

## 6. 成功基準

- **SC1**: プロジェクトルートの有効な`Cargo.toml`ファイルが、Cargo Workspaceを正しく定義し、`runner-debian`と`runner-wslg`をメンバーとしてリストしていること。
- **SC2**: `runner-debian`および`runner-wslg`クレートのディレクトリ構造（例：`runner-debian/src/main.rs`）が正しく作成されていること。
- **SC3**: プロジェクトルートで`cargo build`を実行してもエラーが発生しないこと。
- **SC4**: `cargo run -p runner-debian`を実行すると、正確に「Hello, World!」または「Hello from runner-debian!」と出力されること。
- **SC5**: `cargo run -p runner-wslg`を実行すると、正確に「Hello, World!」または「Hello from runner-wslg!」と出力されること。

---
