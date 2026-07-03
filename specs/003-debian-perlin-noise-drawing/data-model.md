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
    *   `a`: `u8` - アルファ成分 (0-255)。
        # `doc/adr/0004` に従い、ピクセルフォーマットはARGB8888を想定。
        # アルファ値は不透明度を示す。
*   **関係**: N/A
*   **バリデーション**: 各成分は `0` から `255` の範囲。
*   **状態遷移**: N/A

## 3. Framebuffer

DebianのFramebufferデバイス(`/dev/fb0`)を抽象化した構造体。VRAMへの直接アクセスを管理する。

*   **フィールド**:
    *   `path`: `String` - Framebufferデバイスのパス (例: "/dev/fb0")
    *   `file_descriptor`: `RawFd` - Framebufferデバイスのファイルディスクリプタ
    *   `framebuffer_memory`: `FramebufferMemory` - メモリマップされたFramebuffer領域への安全なアクセスを提供する構造体。
    # `core/src/traits.rs` に定義される `VideoMemory` トレイトを
    # 実装する型を想定。これにより、`core` クレートはプラットフォームに
    # 依存しない方法でピクセルデータにアクセスできる。
    # `FramebufferMemory` は `ptr: *mut u8`, `size: usize` などの
    # 低レベルな情報を内部に保持し、安全なスライス操作を提供する。
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
*   **状態遷移**:
    *   `Closed`: デバイスが開かれていない状態。
    *   `Open`: デバイスが開かれ、メモリマップが完了した状態。
        # 状態遷移は `Open` -> `Closed` または `Open` -> `Open` (再マッピングなど)
        # を想定。エラー発生時は `Closed` 状態に戻る。

# 4. FramebufferMemory

メモリマップされたFramebuffer領域への安全なアクセスを提供する抽象化レイヤー。`core` クレートの `VideoMemory` トレイトを実装することを想定。

*   **フィールド**:
    *   `ptr`: `*mut u8` - マップされたメモリ領域の開始ポインタ。
    *   `size`: `usize` - マップされたメモリ領域のサイズ（バイト単位）。
    *   `width`: `u32` - Framebufferの幅（ピクセル単位）。
    *   `height`: `u32` - Framebufferの高さ（ピクセル単位）。
    *   `bytes_per_pixel`: `u32` - 1ピクセルあたりのバイト数。
    *   `line_length`: `u32` - 1ラインあたりのバイト数。
*   **インターフェース (想定)**:
    *   `get_pixel_mut(x, y)`: `Result<&mut PixelData, Error>` - 指定座標のピクセルへのミュータブルな参照を返す。
    *   `write_pixel(x, y, pixel_data)`: `Result<(), Error>` - 指定座標にピクセルデータを書き込む。
    *   `fill_rect(x, y, width, height, color)`: `Result<(), Error>` - 指定範囲を色で塗りつぶす。
    *   `is_valid()`: `bool` - メモリ領域が有効であるかチェックする。
*   **バリデーション**: `ptr` が有効なメモリを指し、`size` が `width * height * bytes_per_pixel` 以上であることを保証する。
*   **状態遷移**: N/A (メモリ領域のライフサイクルは `Framebuffer` 構造体に依存)