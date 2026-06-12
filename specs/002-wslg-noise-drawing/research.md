# Research: WSLgでのノイズ描画機能

## 1. `noise` クレートの利用
- **Decision**: `noise` クレートを使用し、`Perlin` ノイズを生成する。
- **Rationale**: Rustのエコシステムで最も標準的なノイズ生成ライブラリであり、多次元ノイズを効率的に生成できるため。
- **Implementation Detail**: `noise::NoiseFn` トレイトを活用し、座標を渡して値を生成する。

## 2. `minifb` クレートによる描画
- **Decision**: `minifb` を使用して固定サイズのウィンドウを作成し、ピクセルバッファ（`Vec<u32>`）を書き込む。
- **Rationale**: 依存関係が少なく、WSLg環境でもシンプルにピクセル単位の描画検証が可能であるため。
- **Implementation Detail**: `minifb::Window::new` でウィンドウを作成し、`update_with_buffer` で描画を更新する。

## 3. `core` クレートのモジュール構成
- **Decision**: `traits.rs` (インターフェース定義) と `noise.rs` (具象実装) に分離する。
- **Rationale**: 将来的に他のノイズアルゴリズムや描画ロジックを追加した際に、`traits.rs` を変更せずに新しい実装モジュールを追加できるため。
- **Implementation Detail**: `lib.rs` でこれらを `pub mod` として公開し、外部からは `core::traits` および `core::noise` を参照させる。

## 4. `runner-wslg` の統合テスト構成 (ADR0003)
- **Decision**: `runner-wslg/tests/` フォルダに統合テストとして実装する。
- **Rationale**: ユーザーの指示およびADR0003のテストポリシーに基づき、ランナーの健全性検証を統合テストとして扱うため。
- **Implementation Detail**: `tests/noise_drawing.rs` 内で `core` のロジックを注入し、`minifb` ウィンドウが表示されることを検証する。
