---
title: "アクセシビリティ情報の整合性"
---

<span id="accessibility-integrity"></span>

# アクセシビリティ情報の整合性

このルール群では、テンプレートの role、フォーカスの順序、ラベル、要素同士の対応に矛盾がないかを確認します。
ARIA の role を追加したときや、操作に使う要素を変更したときに、関連する属性や要素の関係を見直してください。

たとえば、[role に必要な ARIA 属性](./reference/a11y-role-has-required-aria-props.md)では、role に必要な属性が不足した場合の変更例を確認できます。
悪い例と良い例を比較し、掲載された設定でルールを有効にして、`vp run lint` を実行してください。
[アクセシビリティの全ルール](./accessibility.md)では、ほかの検査も確認できます。

このページに、各ルールの目的・前提・設定と、省略のない悪い例・良い例をまとめています。変更箇所は行の色で示します。コードをコピーすると、ソースコード全体を取得できます。各例に現在の対応範囲も明記しています。


| ルール | 例 | 目的 |
| --- | --- | --- |
| [`a11y/no-role-presentation-on-focusable`](https://vizejs.dev/ja/rules/accessibility-integrity.html#a11y-no-role-presentation-on-focusable) | [悪い](https://vizejs.dev/ja/rules/accessibility-integrity.html#a11y-no-role-presentation-on-focusable-bad) · [良い](https://vizejs.dev/ja/rules/accessibility-integrity.html#a11y-no-role-presentation-on-focusable-good) | フォーカス可能な要素への role="presentation" または role="none" の指定を検出します。 |
| [`a11y/no-static-element-interactions`](https://vizejs.dev/ja/rules/accessibility-integrity.html#a11y-no-static-element-interactions) | [悪い](https://vizejs.dev/ja/rules/accessibility-integrity.html#a11y-no-static-element-interactions-bad) · [良い](https://vizejs.dev/ja/rules/accessibility-integrity.html#a11y-no-static-element-interactions-good) | 操作部品ではない要素へのイベント指定を検出します。 |
| [`a11y/placeholder-label-option`](https://vizejs.dev/ja/rules/accessibility-integrity.html#a11y-placeholder-label-option) | [悪い](https://vizejs.dev/ja/rules/accessibility-integrity.html#a11y-placeholder-label-option-bad) · [良い](https://vizejs.dev/ja/rules/accessibility-integrity.html#a11y-placeholder-label-option-good) | required の select を値が空のプレースホルダー option から始めます。 |
| [`a11y/role-has-required-aria-props`](https://vizejs.dev/ja/rules/accessibility-integrity.html#a11y-role-has-required-aria-props) | [悪い](https://vizejs.dev/ja/rules/accessibility-integrity.html#a11y-role-has-required-aria-props-bad) · [良い](https://vizejs.dev/ja/rules/accessibility-integrity.html#a11y-role-has-required-aria-props-good) | ARIA role が必要とする属性を指定します。 |
| [`a11y/tabindex-no-positive`](https://vizejs.dev/ja/rules/accessibility-integrity.html#a11y-tabindex-no-positive) | [悪い](https://vizejs.dev/ja/rules/accessibility-integrity.html#a11y-tabindex-no-positive-bad) · [良い](https://vizejs.dev/ja/rules/accessibility-integrity.html#a11y-tabindex-no-positive-good) | 通常のフォーカス順序を変える正の tabindex を検出します。 |
| [`a11y/use-list`](https://vizejs.dev/ja/rules/accessibility-integrity.html#a11y-use-list) | [悪い](https://vizejs.dev/ja/rules/accessibility-integrity.html#a11y-use-list-bad) · [良い](https://vizejs.dev/ja/rules/accessibility-integrity.html#a11y-use-list-good) | 箇条書きに見えるテキストをリスト要素で表現します。 |
| [`vue/use-unique-element-ids`](https://vizejs.dev/ja/rules/accessibility-integrity.html#vue-use-unique-element-ids) | [悪い](https://vizejs.dev/ja/rules/accessibility-integrity.html#vue-use-unique-element-ids-bad) · [良い](https://vizejs.dev/ja/rules/accessibility-integrity.html#vue-use-unique-element-ids-good) | 静的 ID の代わりに useId() で一意な ID を生成します。 |

[全ルール](./all.md) · [ルールのオプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)
