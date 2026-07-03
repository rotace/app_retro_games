# Quickstart: Debian環境でのパーリンノイズ描画機能の検証ガイド

## 概要

このガイドは、`runner-debian` クレートが `core` クレートのパーリンノイズ生成機能を利用して、Debian環境のFramebufferにノイズパターンを正しく描画できることを検証する手順を提供します。

## 前提条件

*   Rust開発環境がインストールされているDebian互換システム (例: WSL2上のDebian、または古い物理Debianマシン)。
*   `runner-debian` クレートがFramebufferデバイス (`/dev/fb0`) への読み書き権限を持っていること。通常、これには `root` 権限、または適切なグループ（例: `video` グループ）へのユーザーの追加が必要です。

## セットアップ手順

1.  **プロジェクトのリポジトリをクローンします。**
    ```bash
    git clone <リポジトリURL>
    cd app_retro_games
    ```
2.  **`runner-debian` クレートをビルドします。**
    ```bash
    cargo build --release --bin runner-debian
    ```

## 検証シナリオ1: デフォルトパラメータでのノイズ描画

### 目的

`runner-debian` CLIがデフォルトのFramebuffer設定とノイズパラメータを使用して、パーリンノイズパターンをFramebufferに正しく描画できることを確認します。

### 実行コマンド

```bash
# 権限がない場合は sudo を使用してください
sudo cargo run --release --bin runner-debian draw-noise
```

### 期待される結果

1.  コマンドが正常に実行され、ターミナルに以下のメッセージが表示されます。
    ```
    Perlin noise drawn successfully to framebuffer! Press Enter to exit.
    ```
2.  画面にグレースケールのパーリンノイズパターンが描画されます。
3.  描画されたパターンは、視認可能で途切れることなく、ランダム性のある滑らかなテクスチャに見えます。
4.  Enterキーを押すと、アプリケーションが終了し、画面が元の状態に戻ります。

## 検証シナリオ2: カスタムパラメータでのノイズ描画

### 目的

`runner-debian` CLIがカスタムの幅、高さ、オフセット、スケールパラメータを使用して、パーリンノイズパターンをFramebufferに正しく描画できることを確認します。

### 実行コマンド

```bash
# 権限がない場合は sudo を使用してください
sudo cargo run --release --bin runner-debian draw-noise --width 320 --height 240 --x-offset 100.0 --y-offset 50.0 --scale 0.05
```

### 期待される結果

1.  コマンドが正常に実行され、ターミナルに以下のメッセージが表示されます。
    ```
    Perlin noise drawn successfully to framebuffer! Press Enter to exit.
    ```
2.  指定されたカスタム解像度（例: 320x240）で、オフセットとスケールが適用されたパーリンノイズパターンが画面に描画されます。
3.  描画されたパターンは、デフォルトとは異なる粒度と位置で、指定されたパラメータが効果的に適用されていることを示します。
4.  Enterキーを押すと、アプリケーションが終了し、画面が元の状態に戻ります。

## 関連ドキュメント

*   [データモデル: PerlinNoiseValue, PixelData, Framebuffer](./data-model.md)
*   [契約: Core Perlin Noise Generation Interface](./contracts/core_noise_interface.md)
*   [契約: Runner-Debian Drawing Interface](./contracts/runner_debian_drawing_interface.md)
