---
title: oxlint-plugin-vize アルファ版
description: 新しい Oxlint JS プラグイン ブリッジにより、Vize Patina 診断が Vue SFC の単一の Oxlint 実行に組み込まれます。
---

<!-- Generated translation; source: blog/releases/2026-03-26-oxlint-plugin-vize-alpha.md -->

# `oxlint-plugin-vize` アルファ

<div class="blog-post-meta">
  <span class="blog-meta-chip">
    <span>
      <span class="blog-meta-label">公開</span>
      <span class="blog-meta-value">2026-03-26</span>
    </span>
  </span>
  <a class="blog-author-card" href="https://github.com/ubugeeei">
    <img src="https://github.com/ubugeeei.png" alt="ubugeeei" />
    <span class="blog-author-text">
      <span class="blog-meta-label">著者</span>
      <span class="blog-meta-value">ubugeeei</span>
    </span>
  </a>
</div>

今日は、Vize Patina 用の新しい Oxlint JS プラグイン ブリッジである `oxlint-plugin-vize` の最初のアルファ版を公開します。

目標はシンプルです。[Oxlint](https://oxc.rs/docs/guide/usage/linter) を JavaScript と TypeScript ルールのメイン ランナーとして維持しながら、Vize が同じ実行で Vue 固有の診断に貢献できるようにします。このアルファ版では、Oxlint と Patina のどちらかを選択するのではなく、それらを連携させて機能させることが重要です。

## それは何ですか

`oxlint-plugin-vize` を使用すると、Oxlint の JS プラグイン モデルとルール設定を使用しながら、Oxlint が Vize のネイティブ バインディングを通じて Patina を実行できるようになります。

つまり、1 つの `.oxlintrc.json` で次のようなルールを混在させることができます。

- `no-console` などの Oxlint コア ルール
- Oxlint の組み込み `vue` プラグイン
- `vize/vue/require-v-for-key` などの Vize ルール
- `vize/vue/no-v-html` や `vize/vue/no-duplicate-attributes` などの Patina 対応の Vue 診断

プラグインは `vize` 名前空間を使用し、`settings.vize` から設定を読み取ります。

## このアルファが重要な理由

Patina はすでに Vue テンプレートをよく理解していますが、多くのチームは Oxlint が lint ワークフローの中心に留まり続けることを望んでいます。

このアルファ版は、その形状に向けた最初のステップです。

- 1 つの lint コマンド
- 1つの構成ファイル
- 1 つの出力ストリーム
- Rust ネイティブの JavaScript および TypeScript ルールと Vue テンプレート対応診断

Vue プロジェクトの場合、その組み合わせが重要です。 `v-for` キーの欠落や安全でない `v-html` の使用などのテンプレート ルールは、個別の lint パスや個別のレポート形式を必要とするのではなく、汎用の Oxlint ルールの隣に存在できる必要があります。

## 構成例

```json
{
  "plugins": ["vue"],
  "jsPlugins": ["oxlint-plugin-vize"],
  "settings": {
    "vize": {
      "locale": "en",
      "helpLevel": "none"
    }
  },
  "rules": {
    "no-console": "warn",
    "vize/vue/require-v-for-key": "error",
    "vize/vue/no-v-html": "warn",
    "vize/vue/no-duplicate-attributes": "error"
  }
}
```

アルファ版は現在以下をサポートしています。

- 診断言語の場合は `settings.vize.locale`
- `settings.vize.helpLevel` と `"full"`、`"short"`、または `"none"`
- 下位互換性のための `showHelp`
- `settings.patina` は互換性エイリアスとして使用され、`settings.vize` は正規キーになります

## 仕組み

このブリッジは、Oxlint と戦うのではなく、Oxlint のルールごとの実行モデルを中心に設計されています。

- ファイル上で最初に有効になった Vize ルールは、そのルールに対してのみネイティブの Patina パスを実行します。
- 2 番目の Vize ルールが同じファイルに対して有効になっている場合、プラグインは 1 つの共有フルファイル Patina パスにアップグレードし、その結果を残りの Vize ルールに再利用します。
- ファイルの内容とルールの結果は、Oxlint プロセスの存続期間中、ファイルおよび設定ごとにキャッシュされます。

この設計により、最初のルールを安価に保ちながら、複数の Vize ルールがアクティブになった場合の冗長なネイティブ作業を回避できます。

## 診断と出力

この統合における難しい部分の 1 つは、位置レポートです。

Oxlint の JS プラグイン システムは現在、抽出された Vue スクリプト プログラムから動作しますが、多くの Patina 診断は `<template>` または他の SFC ブロックで発生します。このアルファ版では、`oxlint-plugin-vize` は実際の Vue ブロックと `line:column` を診断メッセージ内でインラインに保持するため、出力は引き続き SFC 内の正しい場所を示します。

リポジトリには、次の混合出力を示す小さな `examples/oxlint-vize` サンプルも含まれています。

- Oxlint コア診断
- Oxlint の組み込み Vue サポート
- 緑青に裏打ちされた Vize 診断

## 現在の制限事項

これはまだアルファ版であり、いくつかの制限事項を明確に指摘することが重要です。

- Oxlint JS プラグインは現在、抽出された Vue スクリプト プログラムに依存しているため、`<script>` または `<script setup>` のないファイルはまだプラグインを呼び出しません。
- Oxlint が元のテンプレート範囲を直接受け入れることができない場合でも、診断アンカーはスクリプト プログラムを指します。
- 最初のアルファ パッケージはノード 24 以降を対象としていました。現在のリリースでは、Node 22 および Node 24+ がサポートされています。
- Oxlint の JS プラグイン サポート自体はまだ進化中であるため、ここでのいくつかの荒削りな点は、Vize のみの動作ではなく上流の制約です。

## なぜ今アルファ版なのか

私は、すべてのエッジケースが完成する前であっても、この統合を早期に人々の手に届けたいと考えていました。

コアの形状はすでに便利だと感じています:

- Vize は Vue 固有の lint インテリジェンスをもたらします
- Oxlint はトップレベルのランナーであり続けます
- 構成面は小さいままです
- パフォーマンス モデルはネイティブ ファーストのままです

テンプレート認識チェックを諦めることなく、より高速な lint スタックを望む Vue ユーザーから実際のフィードバックを得るには、これで十分です。

## 次に何が起こるか

次の手順は簡単です。

- Oxlint がより多くの Vue 対応プラグイン フックを公開するため、テンプレートの場所のマッピングが改善されました。
- プラットフォームのネイティブ バインディングに関するインストールと公開のフローを強化します
- 実際のプロジェクト設定に関するドキュメントと例を拡張します
- Oxlint フォーマッタ内で Patina ヘルプ テキストを表示する方法を引き続き改良します

このアルファは最終状態ではありません。これは、Oxlint と Vize の Vue lint の間の最初の使用可能なブリッジであり、次にどこに行くのかを見るのが楽しみです。
