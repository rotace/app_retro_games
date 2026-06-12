# Implementation Plan: Cargo Workspaceと基本ランナーのセットアップ

**Branch**: `feature/setup-cargo-workspace` | **Date**: 2026-06-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/001-setup-cargo-workspace/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

ADR 0002に基づき、RustのCargo Workspaceを導入し、`core`、`runner-debian`、および`runner-wslg`の3つのクレートを構成する。各ランナーで「Hello, World!」を出力させ、ビルドおよび実行のベースラインを確認する。

## Technical Context

**Language/Version**: Rust (Stable)

**Primary Dependencies**: Cargo Workspace

**Storage**: N/A

**Testing**: `cargo run -p <crate-name>`

**Target Platform**: Linux (Debian), WSLg (Windows)

**Project Type**: CLI/Runner

**Performance Goals**: N/A (Initial setup)

**Constraints**: ADR 0002 準拠（ディレクトリ構造および依存方向）

**Scale/Scope**: 3 crates (`core`, `runner-debian`, `runner-wslg`)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **Rustの使用**: 準拠。
- [x] **ターゲット環境**: Debian/WSLgに対応したランナー構成。
- [x] **2層データ駆動構造**: ADR 0002に従い物理的に分離。
- [x] **依存方向**: `runner-*` -> `core` の単方向依存を徹底。
- [x] **パニック/unsafe禁止**: 基本実装において `unwrap()` 等を排除。
- [x] **言語制約**: ドキュメントおよびコード内コメントを日本語で記述。

## Project Structure

### Documentation (this feature)

```text
specs/001-setup-cargo-workspace/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output
```

### Source Code (repository root)

```text
.
├── Cargo.toml            # Workspace Root
├── core/                 # Core Logic
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
├── runner-debian/        # Debian Runner
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
└── runner-wslg/          # WSLg Runner
    ├── Cargo.toml
    └── src/
        └── main.rs
```

**Structure Decision**: ADR 0002 で定義された標準的な Cargo Workspace 構造を採用。

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | N/A | N/A |
