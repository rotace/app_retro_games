# Implementation Plan: Debian環境でのパーリンノイズ描画機能

**Branch**: `003-debian-perlin-noise-drawing` | **Date**: 2026-07-03 | **Spec**: /home/yasu/workspace/app_retro_games/specs/003-debian-perlin-noise-drawing/spec.md

**Input**: Feature specification from `/specs/003-debian-perlin-noise-drawing/spec.md`

**Note**: This templateは `/speckit.plan` コマンドによって入力されます。実行ワークフローについては `.specify/templates/plan-template.md` を参照してください。

## Summary

`runner-debian` クレートは、`core` クレートのパーリンノイズ生成機能を利用し、古いDebian環境（CUIのみ、Linux Framebuffer直接描画）でその結果をテスト描画機能として画面に表示します。これにより、対象環境でのグラフィックス表示機能の健全性を検証し、将来的なレトロゲームシミュレーションの基盤を確立します。

## Technical Context

**Language/Version**: Rust 1.75+

**Primary Dependencies**: `core` crate (パーリンノイズ生成), `noise` crate (`core`のパーリンノイズ用), `libc` (`runner-debian`のFramebuffer操作用), `ioctls` (`runner-debian`のFramebuffer操作用)

**Storage**: N/A (描画バッファはメモリ上で管理)

**Testing**: `cargo test` (Rustの単体テスト、結合テスト)

**Target Platform**: Linux (古いDebian環境, CUIのみ, Linux Framebuffer (`/dev/fb0`)), Windows (WSLg環境でのシミュレーション)

**Project Type**: library/cli (`core`はライブラリ、`runner-debian`はCLIアプリケーションとして動作)

**Performance Goals**: ノイズパターンは視覚的な確認に十分な速度で描画される必要があります。(例: 30 FPS程度)

**Constraints**:
*   Linux Framebuffer (`/dev/fb0`)への直接ピクセル単位書き込み。
*   古いDebian環境での最小限の依存関係。
*   過度なメモリ消費を避ける。
*   `unwrap()`, `expect()`, `panic!` の禁止。
*   `unsafe` コードの使用は最小限に抑え、安全性の根拠を明記する。

**Scale/Scope**: テスト描画機能としての小規模なグラフィック出力。

## Constitution Check

*GATE: Phase 0の研究前に合格しなければなりません。Phase 1の設計後に再確認します。*

以下の憲法原則に照らして、本計画は準拠しています。

1.  **開発目標とコア制約**:
    *   使用言語: Rust (準拠)
    *   ターゲット環境: 古いDebian PC（GUIなし、ミニマムCUI環境）。Linux Framebuffer (`/dev/fb0`)への直接ピクセル単位書き込み。(準拠)
    *   シミュレーション環境: Windows環境（WSLg）での完全シミュレーション。固定ピクセルウィンドウへの直接ピクセル単位書き込み。(準拠)
2.  **アーキテクチャ原則（2層データ駆動構造）**:
    *   物理パッケージの分離: Cargo Workspace を使用し、コア層 `core` と各ランナー層 `runner-*` を独立したパッケージに分ける。(準拠)
    *   厳格な依存方向: `ランナー層 ➔ コア層`。コア層の `Cargo.toml` がランナー層や外部の具象に依存することは禁止。(準拠)
    *   データ駆動型インターフェース: 各フレームで、入力状態データとターゲットVRAMバッファを、ランナー層からコア層へ単一の関数呼び出しで注入する。(準拠)
    *   ゼロコスト抽象化: レイヤー境界には共通トレイトを導入するが、実行時オーバーヘッドを完全に排除するため、ジェネリクスを用いた静的ディスパッチを徹底する。(準拠)
3.  **品質とエラーハンドリングの絶対原則**:
    *   パニック禁止: `unwrap()`, `expect()`, 明示的な `panic!` は原則として禁止。すべてのエラーは `Result` 型で上位層に返すこと。(準拠)
    *   `unsafe` コードの制限: 原則禁止。Framebuferへの直接書き込みで`unsafe`ブロックが避けられない場合があるため、その際には安全性の根拠（`// SAFETY:` コメント）を明記し、影響範囲を最小限に留めます。(準拠)
4.  **開発と検証の原則**:
    *   静的なノイズ描画ロジックを用いたランナーの健全性検証。(準拠)
    *   AI運用の言語制約: 日本語での思考・応答、ドキュメント、コードコメント。(準拠)
    *   テスト構造: Rustの単体テストと結合テストの配置方針を遵守。(準拠)

## Project Structure

### Documentation (this feature)

```text
specs/003-debian-perlin-noise-drawing/
├── plan.md              # このファイル (/speckit.plan コマンド出力)
├── research.md          # フェーズ0出力 (/speckit.plan コマンド)
├── data-model.md        # フェーズ1出力 (/speckit.plan コマンド)
├── quickstart.md        # フェーズ1出力 (/speckit.plan コマンド)
├── contracts/           # フェーズ1出力 (/speckit.plan コマンド)
└── tasks.md             # フェーズ2出力 (/speckit.tasks コマンド - /speckit.plan では作成されません)
```

### Source Code (リポジトリルート)

```text
.
├── Cargo.toml
├── core/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # パーリンノイズ生成ロジック
│       └── perlin_noise.rs
├── runner-debian/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs         # Debian環境での描画エントリポイント
│       └── framebuffer.rs  # Framebuffer操作ロジック
└── doc/
    └── adr/
```

**構造決定**: Cargo Workspace を採用し、`core` クレートでパーリンノイズ生成ロジックをカプセル化し、`runner-debian` クレートでDebian環境固有の描画ロジック（Framebuffer操作を含む）を実装する2層構造とします。

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | N/A | N/A |
