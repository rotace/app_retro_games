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
    fn stride(&self) -> usize; // 1行あたりのピクセル数
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
