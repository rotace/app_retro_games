# Implementation Plan: WSLgでのノイズ描画機能
 
**Branch**: `feature/wslg-noise-drawing` | **Date**: 2026-06-13 | **Spec**: [specs/002-wslg-noise-drawing/spec.md](../../specs/002-wslg-noise-drawing/spec.md)
 
**Input**: Feature specification from `/specs/002-wslg-noise-drawing/spec.md`
 
**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.
 
## Summary
 
WSLg環境において、パーリンノイズパターンをグラフィカルに描画する機能を実装する。`core`層にプラットフォーム非依存のノイズ生成ロジックを実装し、`runner-wslg`層でそれを`RenderTarget`を通じて描画することで、WSLgでのピクセル単位書き込みの健全性を検証する。
 
## Technical Context
 
**Language/Version**: Rust
 
**Primary Dependencies**: `noise`, `minifb` クレート
 
**Storage**: N/A
 
**Testing**: `cargo test` (ADR0003準拠)
 
**Target Platform**: WSLg (Windows Subsystem for Linux GUI)
 
**Project Type**: desktop-app (CUIレトロゲームシミュレーション)
 
**Performance Goals**: 60 fps
 
**Constraints**: パニック禁止、`unsafe`制限、ドキュメント・コメントの日本語化 (プロジェクト憲法準拠)
 
**Scale/Scope**: WSLgにおける静的なノイズ描画機能の実装
 
## Constitution Check
 
*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*
 
- [x] **Rust使用**: Rustで実装する。
- [x] **Cargo Workspace分離**: `core` と `runner-wslg` を分離。
- [x] **依存方向**: `runner-wslg` ➔ `core` のみ。
- [x] **データ駆動インターフェース**: ADR0004に従い、`InputState` と `RenderTarget` を注入する。
- [x] **ゼロコスト抽象化**: ジェネリクスによる静的ディスパッチを徹底する。
- [x] **パニック禁止**: `unwrap`/`expect` を排除し、`Result` で処理する。
- [x] **`unsafe` 制限**: 必須の場合のみ使用し、`// SAFETY:` コメントを付与する。
- [x] **日本語制約**: 思考・応答・ドキュメント・コメントをすべて日本語とする。
- [x] **テスト構造**: ADR0003に従い、単体テストと結合テストを配置する。
 
## Project Structure
 
### Documentation (this feature)
 
```text
specs/002-wslg-noise-drawing/
├── plan.md              # このファイル
├── research.md          # Phase 0 出力
├── data-model.md        # Phase 1 出力
├── quickstart.md        # Phase 1 出力
├── contracts/           # Phase 1 出力
└── tasks.md             # Phase 2 出力
```
 
### Source Code (repository root)
 
```text
core/
└── src/
    ├── lib.rs          # モジュール宣言
    ├── traits.rs       # GameCore 等のトレイト定義
    └── noise.rs        # noise クレートを用いた実装
runner-wslg/
└── tests/
    └── noise_drawing.rs # WSLg 向け RenderTarget 実装および統合テスト (ADR0003準拠)
```
 
**Structure Decision**: 既存の Cargo Workspace 構造（`core` および `runner-wslg`）をそのまま利用し、機能を拡張する。
 
## Complexity Tracking
 
> **Fill ONLY if Constitution Check has violations that must be justified**
 
| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| なし | - | - |
