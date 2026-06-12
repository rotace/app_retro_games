# Data Model: WSLgでのノイズ描画機能
 
## 1. エンティティ定義
 
### NoiseCore (コア層)
- **役割**: パーリンノイズの生成ロジックを管理する。
- **状態**:
    - `noise_generator`: `noise::Perlin` インスタンス。
    - `offset_x`, `offset_y`: ノイズのサンプリング位置を制御するオフセット（静的な描画の場合は固定）。
- **振る舞い**:
    - `update(input)`: 入力に応じてオフセットを更新する（今回は固定値）。
    - `render(target)`: `RenderTarget` のバッファに対して、パーリンノイズから計算された色値を書き込む。
 
### WslgRenderTarget (ランナー層)
- **役割**: `minifb` 等のウィンドウライブラリが管理するピクセルバッファを `core` 層に提供する。
- **属性**:
    - `buffer`: `&mut [u32]` (ARGB8888形式のピクセル配列)。
    - `width`: 画面幅 (例: 320)。
    - `height`: 画面高さ (例: 240)。
 
## 2. 状態遷移・バリデーション
 
- **バリデーション**:
    - `RenderTarget` のバッファサイズは `width * height` と一致しなければならない。
- **遷移**:
    - 初期化 ➔ メインループ (update ➔ render) ➔ 終了。
