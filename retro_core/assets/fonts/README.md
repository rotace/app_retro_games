# 埋め込みフォント由来

セルフレジの日本語ビットマップ（`src/games/jp_font.rs`）は、次のフォントを
必要文字だけ 16×16 にラスター化したものです。

- **M PLUS Rounded 1c Medium**
  - 作者: Coji Morishita / M+ Fonts Project
  - ライセンス: SIL Open Font License 1.1
  - 入手元: https://fonts.google.com/specimen/M+PLUS+Rounded+1c

元 TTF 自体はリポジトリに同梱していません。再生成する場合は同フォントの
Medium ウェイトを使い、必要文字セットをラスター化してください。
