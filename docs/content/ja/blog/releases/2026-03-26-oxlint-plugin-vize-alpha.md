---
title: oxlint-plugin-vize アルファ版
description: Oxlint の JS プラグインとして Vize の lint を組み込み、Vue SFC の診断を 1 回の Oxlint 実行で確認できるようにします。
---

<!-- Reviewed translation; source: blog/releases/2026-03-26-oxlint-plugin-vize-alpha.md; scope: complete document -->

# `oxlint-plugin-vize` アルファ版

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

`oxlint-plugin-vize` の最初のアルファ版を公開します。Vize のリンターである Patina を、Oxlint の JS プラグインとして動かすためのブリッジです。

目的は単純です。JavaScript と TypeScript のルールは引き続き [Oxlint](https://oxc.rs/docs/guide/usage/linter) が実行し、同じ実行の中で Vize が Vue 固有の診断を追加します。Oxlint と Patina のどちらかを選ぶのではなく、両方を一緒に使えるようにするのが、このアルファ版の狙いです。

## どういうものか

`oxlint-plugin-vize` を使うと、Oxlint の JS プラグインの仕組みとルール設定をそのまま使いながら、Vize のネイティブバインディング経由で Patina を実行できます。

つまり、1 つの `.oxlintrc.json` の中で、次のようなルールを組み合わせられます。

- `no-console` のような Oxlint 本体のルール
- Oxlint に組み込まれている `vue` プラグイン
- `vize/vue/require-v-for-key` のような Vize のルール
- `vize/vue/no-v-html` や `vize/vue/no-duplicate-attributes` のような、Patina による Vue の診断

プラグインは `vize` という名前空間を使い、設定を `settings.vize` から読み込みます。

## このアルファ版の意味

Patina はすでに Vue のテンプレートをよく理解できます。一方で、lint のワークフローの中心には Oxlint を置いておきたい、というチームも多くあります。

このアルファ版は、次の形に向けた最初の一歩です。

- lint のコマンドは 1 つ
- 設定ファイルは 1 つ
- 出力も 1 つ
- Rust 製の JavaScript / TypeScript のルールと、Vue のテンプレートを理解する診断を並べて使う

Vue のプロジェクトでは、この組み合わせに意味があります。`v-for` の key の付け忘れや、安全でない `v-html` の使用といったテンプレートのルールを、汎用的な Oxlint のルールと並べて扱えるべきです。lint を別に実行したり、レポートの形式が分かれたりする必要はありません。

## 設定例

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

このアルファ版は、次の設定に対応しています。

- `settings.vize.locale`: 診断の言語
- `settings.vize.helpLevel`: `"full"`、`"short"`、`"none"` のいずれか
- `showHelp`: 後方互換性のための設定
- `settings.patina`: 互換性のための別名（今後は `settings.vize` を正式なキーとします）

## 仕組み

このブリッジは、Oxlint がルール単位で処理を実行するという仕組みに逆らわず、それに合わせて設計しています。

- あるファイルで最初に有効な Vize のルールは、そのルールだけを対象に、ネイティブの Patina を実行します。
- 同じファイルで 2 つ目の Vize のルールが有効な場合は、ファイル全体を対象にした Patina の実行に切り替え、その結果を残りの Vize のルールでも使い回します。
- ファイルの内容とルールの結果は、ファイルと設定の組み合わせごとに、Oxlint のプロセスが終了するまでキャッシュします。

この設計によって、最初のルールのコストを抑えつつ、複数の Vize のルールが有効な場合にも、ネイティブ側の処理が重複しないようにしています。

## 診断と出力

この連携で難しい点のひとつが、位置の報告です。

Oxlint の JS プラグインは、現在のところ、Vue ファイルから取り出したスクリプトのプログラムを対象に動作します。しかし Patina の診断の多くは、`<template>` などの別のブロックで発生します。このアルファ版では、実際の Vue のブロックと `行:列` を診断メッセージの中に含めることで、SFC の正しい位置にたどり着けるようにしています。

リポジトリには `examples/oxlint-vize` という小さなサンプルもあり、次の 3 種類の出力が混ざった様子を確認できます。

- Oxlint 本体の診断
- Oxlint に組み込まれている Vue 対応の診断
- Patina による Vize の診断

## 現在の制限

これはまだアルファ版です。次の制限があることを、はっきりお伝えしておきます。

- Oxlint の JS プラグインは、Vue ファイルから取り出したスクリプトのプログラムに依存しています。そのため、`<script>` も `<script setup>` もないファイルでは、プラグインがまだ呼び出されません。
- Oxlint が元のテンプレートの範囲をそのまま受け取れない場合、診断の位置はスクリプトのプログラムを指します。
- 最初のアルファ版のパッケージは Node 24 以降を対象としていました。現在のリリースは Node 22 と Node 24 以降に対応しています。
- Oxlint の JS プラグイン機能そのものがまだ発展途中です。そのため、粗い部分のいくつかは Vize だけの問題ではなく、上流の制約によるものです。

## なぜ今アルファ版を出すのか

すべての細かい問題を解決する前でも、この連携を早く使ってもらいたいと考えました。

中心となる形は、すでに役に立つものになっています。

- Vize が Vue 固有の lint の知識を提供する
- 実行の入口は Oxlint のまま
- 設定する項目は少ないまま
- 性能はネイティブ実行を前提にしたまま

テンプレートを理解するチェックを手放さずに、より速い lint の環境を求めている Vue ユーザーから、実際のフィードバックをもらい始めるには十分です。

## 今後の予定

次に取り組むことは明確です。

- Oxlint が Vue を意識したプラグイン用のフックを公開するのに合わせて、テンプレートの位置の対応付けを改善する
- プラットフォームごとのネイティブバインディングについて、インストールと公開の流れを安定させる
- 実際のプロジェクト構成に合わせたドキュメントとサンプルを増やす
- Oxlint のフォーマッタの中で、Patina のヘルプをどう見せるかを改善し続ける

このアルファ版は完成形ではありません。Oxlint と、Vize の Vue 向け lint をつなぐ、最初の実用的なブリッジです。ここからどう発展していくのか、楽しみにしています。
