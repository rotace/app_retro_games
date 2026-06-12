# Research: WSLgでのノイズ描画機能
 
## 1. ノイズライブラリの選定
 
- **Decision**: `noise` クレート (https://crates.io/crates/noise) を使用する。
- **Rationale**: Rust におけるノイズ生成の標準的なライブラリであり、パーリンノイズ（Perlin Noise）を含む多様なノイズアルゴリズムをサポートしている。API がシンプルで、今回の要件である「固定値でのパーリンノイズ生成」を容易に実現できる。
- **Alternatives considered**: 
    - 自前実装: シンプルなパーリンノイズであれば可能だが、車輪の再発明となり、テスト済みのライブラリを使用する方が信頼性が高い。
    - 他の軽量ライブラリ: `noise` クレートが十分軽量であり、機能的に十分であるため、他を検討する必要はない。
 
## 2. WSLg での描画手法
 
- **Decision**: WSLg は X11 または Wayland サーバーとして動作するため、シンプルなウィンドウシステムライブラリ（例: `sdl2` または `minifb`）を使用してピクセルバッファを転送し、表示する。
- **Rationale**: 
    - `minifb` は依存関係が少なく、単一のピクセルバッファをウィンドウに表示することに特化しており、今回の「静的なノイズ描画」による健全性検証という目的に最適である。
    - `sdl2` は高機能だが、外部ライブラリ（Cライブラリ）のインストールが必要であり、WSLg 環境でのセットアップコストが高い。
- **Alternatives considered**: 
    - Linux Framebuffer (`/dev/fb0`): `runner-debian` では使用するが、WSLg 環境では仮想ディスプレイを使用するため、直接書き込みよりもウィンドウライブラリを通じた描画が一般的である。
 
## 3. ADR0004 の適用プラン
 
- **InputState**: 今回は静的な描画のため、`InputState` はデフォルト値で注入する。
- **RenderTarget**: `minifb` のバッファをラップした `WslgRenderTarget` を実装し、`core` 層の `render` 関数に渡す。
- **GameCore**: `NoiseCore` という構造体を実装し、`update` でノイズの状態を更新し、`render` で `RenderTarget` にピクセルを書き込む。
