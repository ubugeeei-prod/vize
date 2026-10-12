---
title: "ページ構造のアクセシビリティ"
---

<span id="accessibility-structure"></span>

# ページ構造のアクセシビリティ

このルール群では、見出しの内容と階層、ランドマーク、フォームのラベル、iframe の説明、
画像の代替テキスト、音声・動画の字幕、操作する要素のフォーカスを確認します。
コンポーネントに新しい領域や埋め込みコンテンツ、画像、メディアプレイヤーを追加するときに、ページ構造を見直してください。

たとえば、[見出しの階層](./reference/a11y-heading-levels.md)では、見出しのレベルを飛ばした場合の変更例を確認できます。
悪い例と良い例を比較し、掲載された設定でルールを有効にして、`vp run lint` を実行してください。
[アクセシビリティの全ルール](./accessibility.md)では、関連するルールも確認できます。

このページに、各ルールの目的・前提・設定と、省略のない悪い例・良い例をまとめています。変更箇所は行の色で示します。コードをコピーすると、ソースコード全体を取得できます。各例に現在の対応範囲も明記しています。


| ルール | 例 | 目的 |
| --- | --- | --- |
| [`a11y/heading-has-content`](https://vizejs.dev/ja/rules/accessibility-structure.html#a11y-heading-has-content) | [悪い](https://vizejs.dev/ja/rules/accessibility-structure.html#a11y-heading-has-content-bad) · [良い](https://vizejs.dev/ja/rules/accessibility-structure.html#a11y-heading-has-content-good) | 見出しに支援技術で読める内容を用意します。 |
| [`a11y/heading-levels`](https://vizejs.dev/ja/rules/accessibility-structure.html#a11y-heading-levels) | [悪い](https://vizejs.dev/ja/rules/accessibility-structure.html#a11y-heading-levels-bad) · [良い](https://vizejs.dev/ja/rules/accessibility-structure.html#a11y-heading-levels-good) | 見出しの階層を飛ばした指定を検出します。 |
| [`a11y/iframe-has-title`](https://vizejs.dev/ja/rules/accessibility-structure.html#a11y-iframe-has-title) | [悪い](https://vizejs.dev/ja/rules/accessibility-structure.html#a11y-iframe-has-title-bad) · [良い](https://vizejs.dev/ja/rules/accessibility-structure.html#a11y-iframe-has-title-good) | iframe に内容を説明する title を指定します。 |
| [`a11y/img-alt`](https://vizejs.dev/ja/rules/accessibility-structure.html#a11y-img-alt) | [悪い](https://vizejs.dev/ja/rules/accessibility-structure.html#a11y-img-alt-bad) · [良い](https://vizejs.dev/ja/rules/accessibility-structure.html#a11y-img-alt-good) | 画像に alt 属性を指定します。装飾画像は空の alt を使います。 |
| [`a11y/interactive-supports-focus`](https://vizejs.dev/ja/rules/accessibility-structure.html#a11y-interactive-supports-focus) | [悪い](https://vizejs.dev/ja/rules/accessibility-structure.html#a11y-interactive-supports-focus-bad) · [良い](https://vizejs.dev/ja/rules/accessibility-structure.html#a11y-interactive-supports-focus-good) | 操作可能な role の要素をフォーカス可能にします。 |
| [`a11y/label-has-for`](https://vizejs.dev/ja/rules/accessibility-structure.html#a11y-label-has-for) | [悪い](https://vizejs.dev/ja/rules/accessibility-structure.html#a11y-label-has-for-bad) · [良い](https://vizejs.dev/ja/rules/accessibility-structure.html#a11y-label-has-for-good) | label を対象のフォーム部品と関連付けます。 |
| [`a11y/landmark-roles`](https://vizejs.dev/ja/rules/accessibility-structure.html#a11y-landmark-roles) | [悪い](https://vizejs.dev/ja/rules/accessibility-structure.html#a11y-landmark-roles-bad) · [良い](https://vizejs.dev/ja/rules/accessibility-structure.html#a11y-landmark-roles-good) | ランドマーク role の配置と重複を検査します。 |
| [`a11y/media-has-caption`](https://vizejs.dev/ja/rules/accessibility-structure.html#a11y-media-has-caption) | [悪い](https://vizejs.dev/ja/rules/accessibility-structure.html#a11y-media-has-caption-bad) · [良い](https://vizejs.dev/ja/rules/accessibility-structure.html#a11y-media-has-caption-good) | 音声・動画に字幕を用意します。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)
