---
title: "操作とフォーカスのアクセシビリティ"
---

<span id="accessibility-interactions"></span>

# 操作とフォーカスのアクセシビリティ

このルール群では、キーボード操作に関わる指定、フォーカス、マウスのイベント処理、支援技術から隠す要素、要素同士の参照を確認します。
独自の操作部品を追加したり、表示の切り替えやイベントの処理を変更したりするときに確認してください。

たとえば、[フォーカスと aria-hidden の関係](./reference/a11y-no-aria-hidden-on-focusable.md)では、フォーカスできる要素を支援技術から隠してしまう指定の例を確認できます。
悪い例と良い例を比較し、掲載された設定を使って、`vp run lint` を実行してください。
[アクセシビリティの全ルール](./accessibility.md)では、関連するルールも確認できます。

このページに、各ルールの目的・前提・設定と、省略のない悪い例・良い例をまとめています。変更箇所は行の色で示します。コードをコピーすると、ソースコード全体を取得できます。各例に現在の対応範囲も明記しています。


| ルール | 例 | 目的 |
| --- | --- | --- |
| [`a11y/mouse-events-have-key-events`](https://vizejs.dev/ja/rules/accessibility-interactions.html#a11y-mouse-events-have-key-events) | [悪い](https://vizejs.dev/ja/rules/accessibility-interactions.html#a11y-mouse-events-have-key-events-bad) · [良い](https://vizejs.dev/ja/rules/accessibility-interactions.html#a11y-mouse-events-have-key-events-good) | マウス操作と対応する focus / blur 操作を用意します。 |
| [`a11y/no-access-key`](https://vizejs.dev/ja/rules/accessibility-interactions.html#a11y-no-access-key) | [悪い](https://vizejs.dev/ja/rules/accessibility-interactions.html#a11y-no-access-key-bad) · [良い](https://vizejs.dev/ja/rules/accessibility-interactions.html#a11y-no-access-key-good) | 環境のショートカットと衝突し得る accesskey を検出します。 |
| [`a11y/no-aria-hidden-on-focusable`](https://vizejs.dev/ja/rules/accessibility-interactions.html#a11y-no-aria-hidden-on-focusable) | [悪い](https://vizejs.dev/ja/rules/accessibility-interactions.html#a11y-no-aria-hidden-on-focusable-bad) · [良い](https://vizejs.dev/ja/rules/accessibility-interactions.html#a11y-no-aria-hidden-on-focusable-good) | フォーカス可能な要素を aria-hidden で支援技術から隠す指定を検出します。 |
| [`a11y/no-autofocus`](https://vizejs.dev/ja/rules/accessibility-interactions.html#a11y-no-autofocus) | [悪い](https://vizejs.dev/ja/rules/accessibility-interactions.html#a11y-no-autofocus-bad) · [良い](https://vizejs.dev/ja/rules/accessibility-interactions.html#a11y-no-autofocus-good) | 意図せずフォーカスを移動させる autofocus を検出します。 |
| [`a11y/no-distracting-elements`](https://vizejs.dev/ja/rules/accessibility-interactions.html#a11y-no-distracting-elements) | [悪い](https://vizejs.dev/ja/rules/accessibility-interactions.html#a11y-no-distracting-elements-bad) · [良い](https://vizejs.dev/ja/rules/accessibility-interactions.html#a11y-no-distracting-elements-good) | marquee や blink などの注意をそらす要素を検出します。 |
| [`a11y/no-i-for-icon`](https://vizejs.dev/ja/rules/accessibility-interactions.html#a11y-no-i-for-icon) | [悪い](https://vizejs.dev/ja/rules/accessibility-interactions.html#a11y-no-i-for-icon-bad) · [良い](https://vizejs.dev/ja/rules/accessibility-interactions.html#a11y-no-i-for-icon-good) | アイコン用の i 要素を検出し、意味に合う要素を勧めます。 |
| [`a11y/no-redundant-roles`](https://vizejs.dev/ja/rules/accessibility-interactions.html#a11y-no-redundant-roles) | [悪い](https://vizejs.dev/ja/rules/accessibility-interactions.html#a11y-no-redundant-roles-bad) · [良い](https://vizejs.dev/ja/rules/accessibility-interactions.html#a11y-no-redundant-roles-good) | 要素本来の意味と重複する ARIA role を検出します。 |
| [`a11y/no-refer-to-non-existent-id`](https://vizejs.dev/ja/rules/accessibility-interactions.html#a11y-no-refer-to-non-existent-id) | [悪い](https://vizejs.dev/ja/rules/accessibility-interactions.html#a11y-no-refer-to-non-existent-id-bad) · [良い](https://vizejs.dev/ja/rules/accessibility-interactions.html#a11y-no-refer-to-non-existent-id-good) | 文書内に存在しない ID への参照を検出します。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)
