# 4. interface-design

Date: 2026-06-12

## Status

Accepted

## Context

プロジェクト憲法 第2項（アーキテクチャ原則）において、コア層（`core`）とランナー層（`runner-*`）の分離、および「データ駆動型インターフェース」と「ゼロコスト抽象化」の徹底が規定されています。

`core` はOSや描画ライブラリなどの具象プラットフォームに依存してはならず、各フレームの処理はランナー層から「入力状態データ」と「VRAMバッファ」を注入することで成立させる必要があります。また、この境界で発生する抽象化において、ランタイムのパフォーマンス（特に古いDebian PCのミニマムCUI環境におけるCPU/メモリリソース）への影響を完全に排除する（ゼロコスト）必要があります。

## Decision


### 1. インターフェース定義

コア層（`core`）に、ランナーから注入される状態およびバッファを表すデータ構造と、コア層のメインロジックを駆動するための共通トレイトを定義します。

- **入力データ構造 (`InputState`)**: キー入力やタイマーなどの状態を保持するプレーンな構造体 (POD)。
- **出力バッファ (`VramBuffer`)**: 描画対象となる固定長のピクセルバッファ、またはバッファへのアクセスを提供する排他スライス。
- **ゲームループトレイト (`GameLoop`)**: 各ランナーがインクルードして静的ディスパッチを行うための境界インターフェース。

### 2. 静的ディスパッチ（ジェネリクス）の徹底

トレイトオブジェクト（`dyn Trait`）による動的ディスパッチ（vtable 経由の呼び出しオーバーヘッド）を禁止し、コンパイル時モノモーフィゼーション（Inliningの最適化）を効かせるため、**ジェネリクスと `impl Trait` による静的ディスパッチ**を徹底します。

### 3. メモリレイアウトの固定

CUIレトロゲームとしての制約（Linux Framebufferへの直接書き込み）を満たすため、VRAMバッファはヒープ配置（`Vec`）ではなく、固定長の配列またはスライスとして扱い、キャッシュ効率を最大化します。

### コア層 (`core/src/lib.rs`)

```rust
/// ランナーから注入される入力データ
#[derive(Debug, Clone, Copy, Default)]
pub struct InputState {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub action: bool,
}

/// 描画対象となるVRAMバッファの抽象化
pub trait RenderTarget {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn buffer_mut(&mut self) -> &mut [u32]; // ARGB8888 想定
}

/// ゲームの実行ロジックを定義するトレイト
pub trait GameCore {
    fn update(&mut self, input: &InputState);
    fn render<R: RenderTarget>(&self, target: &mut R);
}

/// ゼロコスト抽象化されたフレーム実行関数
/// ジェネリクスにより、コンパイル時に具象型に展開・インライン化される
#[inline(always)]
pub fn tick_frame<G: GameCore, R: RenderTarget>(
    core: &mut G,
    render_target: &mut R,
    input: &InputState,
) {
    core.update(input);
    core.render(render_target);
}

```

### ランナー層での利用例 (`runner-linux/src/main.rs`)

```rust
use core::{InputState, RenderTarget, GameCore, tick_frame};

// Linux Framebuffer 向けの具象ターゲット
struct FramebufferTarget<'a> {
    fb_slice: &'a mut [u32],
    width: usize,
    height: usize,
}

impl<'a> RenderTarget for FramebufferTarget<'a> {
    fn width(&self) -> usize { self.width }
    fn height(&self) -> usize { self.height }
    fn buffer_mut(&mut self) -> &mut [u32] { self.fb_slice }
}

fn main() {
    // 実際のハードウェア・初期化処理（Linux FB / WSLg等）
    let mut fake_vram = vec![0u32; 320 - 240];
    
    // コアのインスタンス化 (具体的なゲーム実装)
    // let mut game = MyGame::new();
    
    loop {
        // 1. 各プラットフォーム固有の方法で入力を取得
        let input = InputState { up: true, ..Default::default() };
        
        // 2. ターゲットバッファの構築
        let mut target = FramebufferTarget {
            fb_slice: &mut fake_vram,
            width: 320,
            height: 240,
        };
        
        // 3. 単一の関数呼び出しでデータを注入し駆動（静的ディスパッチ）
        // tick_frame(&mut game, &mut target, &input);
        
        break; // 実際はメインループ
    }
}

```


## Consequences

### Positive

- **ゼロコストの実現**: ジェネリクスと `#[inline(always)]` を活用することで、関数呼び出しのオーバーヘッドが完全に消滅し、ランナーとコアの境界がコンパイル時に最適化されます。
- **完全なデータ駆動**: コア側は状態を保持せず（あるいは自身のゲーム状態のみを保持）、ランナー側から与えられた `InputState` と `RenderTarget` だけで完結するため、テストが極めて容易になります（モックのインジェクションが容易）。
- **依存関係の強制分離**: コア層は `RenderTarget` トレイトを提供するだけであり、具体的なOSのファイル記述子やウィンドウハンドルを一切知る必要がありません。

### Negative

- **ビルド時間の増加**: ジェネリクスによるモノモーフィゼーションを行うため、ランナーごとにコードが複製され、コンパイル時間が若干増加します。ただし、今回のWorkspace構造（コア1つに対して独立した少数ランナー）であれば影響は軽微です。
- **バイナリサイズの増加**: 静的展開によるバイナリの肥大化リスクがありますが、CUIレトロゲームの規模感であればメモリ制約（古いDebian PC）を突破するほどにはなりません。
