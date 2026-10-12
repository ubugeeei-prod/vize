---
title: "アクセシビリティの基本"
---

<span id="accessibility-core"></span>

# アクセシビリティの基本

このルール群では、画像の代替テキスト、リンクの内容、ARIA 属性、キーボード操作、フォームのラベルなど、
Vue テンプレートの基本的なアクセシビリティ情報を確認します。
画像やリンク、入力欄をコンポーネントに追加するときは、ここから確認してください。

たとえば、入力欄を追加する前に[フォーム部品のラベル](./reference/a11y-form-control-has-label.md)を開き、
悪い例と良い例を比較します。掲載された Vite+ の設定でルールを有効にし、`vp run lint` を実行してください。
[アクセシビリティの全ルール](./accessibility.md)では、関連するルールも確認できます。

このページに、各ルールの目的・前提・設定と、省略のない悪い例・良い例をまとめています。変更箇所は行の色で示します。コードをコピーすると、ソースコード全体を取得できます。各例に現在の対応範囲も明記しています。


| ルール | 例 | 目的 |
| --- | --- | --- |
| [`a11y/alt-text`](https://vizejs.dev/ja/rules/accessibility-core.html#a11y-alt-text) | [悪い](https://vizejs.dev/ja/rules/accessibility-core.html#a11y-alt-text-bad) · [良い](https://vizejs.dev/ja/rules/accessibility-core.html#a11y-alt-text-good) | 画像などのメディアに代替テキストを用意します。 |
| [`a11y/anchor-has-content`](https://vizejs.dev/ja/rules/accessibility-core.html#a11y-anchor-has-content) | [悪い](https://vizejs.dev/ja/rules/accessibility-core.html#a11y-anchor-has-content-bad) · [良い](https://vizejs.dev/ja/rules/accessibility-core.html#a11y-anchor-has-content-good) | リンクに支援技術で読める内容を用意します。 |
| [`a11y/anchor-is-valid`](https://vizejs.dev/ja/rules/accessibility-core.html#a11y-anchor-is-valid) | [悪い](https://vizejs.dev/ja/rules/accessibility-core.html#a11y-anchor-is-valid-bad) · [良い](https://vizejs.dev/ja/rules/accessibility-core.html#a11y-anchor-is-valid-good) | リンクの href に有効な移動先を指定します。 |
| [`a11y/aria-props`](https://vizejs.dev/ja/rules/accessibility-core.html#a11y-aria-props) | [悪い](https://vizejs.dev/ja/rules/accessibility-core.html#a11y-aria-props-bad) · [良い](https://vizejs.dev/ja/rules/accessibility-core.html#a11y-aria-props-good) | 存在しない ARIA 属性を検出します。 |
| [`a11y/aria-role`](https://vizejs.dev/ja/rules/accessibility-core.html#a11y-aria-role) | [悪い](https://vizejs.dev/ja/rules/accessibility-core.html#a11y-aria-role-bad) · [良い](https://vizejs.dev/ja/rules/accessibility-core.html#a11y-aria-role-good) | 抽象ロールを除く、有効な ARIA role を指定します。 |
| [`a11y/aria-unsupported-elements`](https://vizejs.dev/ja/rules/accessibility-core.html#a11y-aria-unsupported-elements) | [悪い](https://vizejs.dev/ja/rules/accessibility-core.html#a11y-aria-unsupported-elements-bad) · [良い](https://vizejs.dev/ja/rules/accessibility-core.html#a11y-aria-unsupported-elements-good) | ARIA 属性を使用できない要素への指定を検出します。 |
| [`a11y/click-events-have-key-events`](https://vizejs.dev/ja/rules/accessibility-core.html#a11y-click-events-have-key-events) | [悪い](https://vizejs.dev/ja/rules/accessibility-core.html#a11y-click-events-have-key-events-bad) · [良い](https://vizejs.dev/ja/rules/accessibility-core.html#a11y-click-events-have-key-events-good) | クリックで操作する要素にキーボード操作も用意します。 |
| [`a11y/form-control-has-label`](https://vizejs.dev/ja/rules/accessibility-core.html#a11y-form-control-has-label) | [悪い](https://vizejs.dev/ja/rules/accessibility-core.html#a11y-form-control-has-label-bad) · [良い](https://vizejs.dev/ja/rules/accessibility-core.html#a11y-form-control-has-label-good) | フォーム部品に関連付けられたラベルを用意します。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)
