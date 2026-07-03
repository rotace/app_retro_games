# Data Model: Debian環境でのパーリンノイズ描画機能

## 1. PerlinNoiseValue

パーリンノイズアルゴリズムによって生成される単一のノイズ値。

*   **フィールド**:
    *   `value`: `f32` - 0.0から1.0の範囲のノイズ値。
*   **関係**: N/A
*   **バリデーション**: `0.0 <= value <= 1.0`
*   **状態遷移**: N/A

## 2. PixelData

描画バッファ内の単一ピクセルの色データ。Debian Framebufferは通常、RGB565, RGB888, ARGB8888などのフォーマットをサポートしているが、ここではシンプルにグレースケール値を想定。

*   **フィールド**:
    *   `r`: `u8` - 赤成分 (0-255)
    *   `g`: `u8` - 緑成分 (0-255)
    *   `b`: `u8` - 青成分 (0-255)
    *   `a`: `u8` - アルファ成分 (0-255) (必要に応じて。今回はグレースケール描画のため0xFF固定を想定)
*   **関係**: N/A
*   **バリデーション**: 各成分は `0` から `255` の範囲。
*   **状態遷移**: N/A

## 3. Framebuffer

DebianのFramebufferデバイス(`/dev/fb0`)を抽象化した構造体。VRAMへの直接アクセスを管理する。

*   **フィールド**:
    *   `path`: `String` - Framebufferデバイスのパス (例: "/dev/fb0")
    *   `file_descriptor`: `RawFd` - Framebufferデバイスのファイルディスクリプタ
    *   `buffer`: `*mut u8` - メモリマップされたFramebufferの開始ポインタ
    *   `size`: `usize` - メモリマップされたFramebufferのサイズ (バイト単位)
    *   `width`: `u32` - Framebufferの幅 (ピクセル単位)
    *   `height`: `u32` - Framebufferの高さ (ピクセル単位)
    *   `bytes_per_pixel`: `u32` - 1ピクセルあたりのバイト数 (例: RGB8888なら4)
    *   `line_length`: `u32` - 1ラインあたりのバイト数
*   **関係**: `PixelData` をFramebufferに書き込む。
*   **バリデーション**:
    *   `file_descriptor` は有効なFDであること。
    *   `buffer` は有効なメモリ領域を指していること。
    *   `width`, `height`, `bytes_per_pixel`, `line_length` は有効なデバイス情報と一致すること。
*   **状態遷移**: N/A (オープン、クローズ、描画)
