# Contract: GameCore Interface

## 概要
`core` 層と `runner` 層の間のデータ交換ルールを定義する。ADR0004に従い、静的ディスパッチ（ジェネリクス）を用いる。

## インターフェース定義

### `GameCore` トレイト
コアロジックが実装すべきインターフェース。

- **メソッド**: `tick_frame`
    - **引数**:
        - `input: &InputState`: 現在の入力状態。
        - `target: &mut dyn RenderTarget`: 書き込み先の描画ターゲット。
    - **戻り値**: `Result<(), CoreError>`
    - **挙動**: 1フレーム分の更新処理を行い、`target` のバッファに描画結果を書き込む。

### `RenderTarget` トレイト
ランナー側が実装し、コア側に提供する描画バッファのインターフェース。

- **メソッド**: `write_pixel`
    - **引数**: `(x: usize, y: usize, color: u32)`
    - **挙動**: 指定した座標に色を書き込む。
- **メソッド**: `get_dimensions`
    - **戻り値**: `(width: usize, height: usize)`

## 制約
- `core` 層は `RenderTarget` の具象実装（`minifb` 等）に依存してはならない。
- すべての操作は `Result` 型でエラーを返し、パニックを禁止する。
