---
title: アクセシビリティ ルール
---

# アクセシビリティ ルール

ルール名から、目的・重大度・適用範囲・設定・悪い例・良い例を確認できます。全ルールの一覧に例と現在の対応範囲を同じページでまとめています。

Vite+ では `lint.vize.rules` に設定し、`vp run lint` を実行します。型が必要なルールや、専用ファイル・追加設定が必要なルールは個別ページの前提を確認してください。

| ルール | 例 | 目的 |
| --- | --- | --- |
| [`a11y/alt-text`](./all.md#a11y-alt-text) | [悪い例](./all.md#a11y-alt-text-bad) · [良い例](./all.md#a11y-alt-text-good) | 画像などのメディアに代替テキストを用意します。 |
| [`a11y/anchor-has-content`](./all.md#a11y-anchor-has-content) | [悪い例](./all.md#a11y-anchor-has-content-bad) · [良い例](./all.md#a11y-anchor-has-content-good) | リンクに支援技術で読める内容を用意します。 |
| [`a11y/anchor-is-valid`](./all.md#a11y-anchor-is-valid) | [悪い例](./all.md#a11y-anchor-is-valid-bad) · [良い例](./all.md#a11y-anchor-is-valid-good) | リンクの href に有効な移動先を指定します。 |
| [`a11y/aria-props`](./all.md#a11y-aria-props) | [悪い例](./all.md#a11y-aria-props-bad) · [良い例](./all.md#a11y-aria-props-good) | 存在しない ARIA 属性を検出します。 |
| [`a11y/aria-role`](./all.md#a11y-aria-role) | [悪い例](./all.md#a11y-aria-role-bad) · [良い例](./all.md#a11y-aria-role-good) | 有効で抽象的ではない ARIA role を指定します。 |
| [`a11y/aria-unsupported-elements`](./all.md#a11y-aria-unsupported-elements) | [悪い例](./all.md#a11y-aria-unsupported-elements-bad) · [良い例](./all.md#a11y-aria-unsupported-elements-good) | ARIA 属性を使用できない要素への指定を検出します。 |
| [`a11y/click-events-have-key-events`](./all.md#a11y-click-events-have-key-events) | [悪い例](./all.md#a11y-click-events-have-key-events-bad) · [良い例](./all.md#a11y-click-events-have-key-events-good) | クリックで操作する要素にキーボード操作も用意します。 |
| [`a11y/form-control-has-label`](./all.md#a11y-form-control-has-label) | [悪い例](./all.md#a11y-form-control-has-label-bad) · [良い例](./all.md#a11y-form-control-has-label-good) | フォーム部品に関連付けられたラベルを用意します。 |
| [`a11y/heading-has-content`](./all.md#a11y-heading-has-content) | [悪い例](./all.md#a11y-heading-has-content-bad) · [良い例](./all.md#a11y-heading-has-content-good) | 見出しに支援技術で読める内容を用意します。 |
| [`a11y/heading-levels`](./all.md#a11y-heading-levels) | [悪い例](./all.md#a11y-heading-levels-bad) · [良い例](./all.md#a11y-heading-levels-good) | 見出しの階層を飛ばした指定を検出します。 |
| [`a11y/iframe-has-title`](./all.md#a11y-iframe-has-title) | [悪い例](./all.md#a11y-iframe-has-title-bad) · [良い例](./all.md#a11y-iframe-has-title-good) | iframe に内容を説明する title を指定します。 |
| [`a11y/img-alt`](./all.md#a11y-img-alt) | [悪い例](./all.md#a11y-img-alt-bad) · [良い例](./all.md#a11y-img-alt-good) | 画像に alt 属性を指定します。装飾画像は空の alt を使います。 |
| [`a11y/interactive-supports-focus`](./all.md#a11y-interactive-supports-focus) | [悪い例](./all.md#a11y-interactive-supports-focus-bad) · [良い例](./all.md#a11y-interactive-supports-focus-good) | 操作可能な role の要素をフォーカス可能にします。 |
| [`a11y/label-has-for`](./all.md#a11y-label-has-for) | [悪い例](./all.md#a11y-label-has-for-bad) · [良い例](./all.md#a11y-label-has-for-good) | label を対象のフォーム部品と関連付けます。 |
| [`a11y/landmark-roles`](./all.md#a11y-landmark-roles) | [悪い例](./all.md#a11y-landmark-roles-bad) · [良い例](./all.md#a11y-landmark-roles-good) | ランドマーク role の配置と重複を検査します。 |
| [`a11y/media-has-caption`](./all.md#a11y-media-has-caption) | [悪い例](./all.md#a11y-media-has-caption-bad) · [良い例](./all.md#a11y-media-has-caption-good) | 音声・動画に字幕を用意します。 |
| [`a11y/mouse-events-have-key-events`](./all.md#a11y-mouse-events-have-key-events) | [悪い例](./all.md#a11y-mouse-events-have-key-events-bad) · [良い例](./all.md#a11y-mouse-events-have-key-events-good) | マウス操作と対応する focus / blur 操作を用意します。 |
| [`a11y/no-access-key`](./all.md#a11y-no-access-key) | [悪い例](./all.md#a11y-no-access-key-bad) · [良い例](./all.md#a11y-no-access-key-good) | 環境のショートカットと衝突し得る accesskey を検出します。 |
| [`a11y/no-aria-hidden-on-focusable`](./all.md#a11y-no-aria-hidden-on-focusable) | [悪い例](./all.md#a11y-no-aria-hidden-on-focusable-bad) · [良い例](./all.md#a11y-no-aria-hidden-on-focusable-good) | フォーカス可能な要素を aria-hidden で隠した指定を検出します。 |
| [`a11y/no-autofocus`](./all.md#a11y-no-autofocus) | [悪い例](./all.md#a11y-no-autofocus-bad) · [良い例](./all.md#a11y-no-autofocus-good) | 意図せずフォーカスを移動させる autofocus を検出します。 |
| [`a11y/no-distracting-elements`](./all.md#a11y-no-distracting-elements) | [悪い例](./all.md#a11y-no-distracting-elements-bad) · [良い例](./all.md#a11y-no-distracting-elements-good) | marquee や blink などの注意をそらす要素を検出します。 |
| [`a11y/no-i-for-icon`](./all.md#a11y-no-i-for-icon) | [悪い例](./all.md#a11y-no-i-for-icon-bad) · [良い例](./all.md#a11y-no-i-for-icon-good) | アイコン用の i 要素を検出し、意味に合う要素を勧めます。 |
| [`a11y/no-redundant-roles`](./all.md#a11y-no-redundant-roles) | [悪い例](./all.md#a11y-no-redundant-roles-bad) · [良い例](./all.md#a11y-no-redundant-roles-good) | 要素本来の意味と重複する ARIA role を検出します。 |
| [`a11y/no-refer-to-non-existent-id`](./all.md#a11y-no-refer-to-non-existent-id) | [悪い例](./all.md#a11y-no-refer-to-non-existent-id-bad) · [良い例](./all.md#a11y-no-refer-to-non-existent-id-good) | 文書内に存在しない ID への参照を検出します。 |
| [`a11y/no-role-presentation-on-focusable`](./all.md#a11y-no-role-presentation-on-focusable) | [悪い例](./all.md#a11y-no-role-presentation-on-focusable-bad) · [良い例](./all.md#a11y-no-role-presentation-on-focusable-good) | フォーカス可能な要素の意味を presentation で消した指定を検出します。 |
| [`a11y/no-static-element-interactions`](./all.md#a11y-no-static-element-interactions) | [悪い例](./all.md#a11y-no-static-element-interactions-bad) · [良い例](./all.md#a11y-no-static-element-interactions-good) | 操作部品ではない要素へのイベント指定を検出します。 |
| [`a11y/placeholder-label-option`](./all.md#a11y-placeholder-label-option) | [悪い例](./all.md#a11y-placeholder-label-option-bad) · [良い例](./all.md#a11y-placeholder-label-option-good) | select のプレースホルダー option に disabled または hidden を指定します。 |
| [`a11y/role-has-required-aria-props`](./all.md#a11y-role-has-required-aria-props) | [悪い例](./all.md#a11y-role-has-required-aria-props-bad) · [良い例](./all.md#a11y-role-has-required-aria-props-good) | ARIA role が必要とする属性を指定します。 |
| [`a11y/tabindex-no-positive`](./all.md#a11y-tabindex-no-positive) | [悪い例](./all.md#a11y-tabindex-no-positive-bad) · [良い例](./all.md#a11y-tabindex-no-positive-good) | 通常のフォーカス順序を変える正の tabindex を検出します。 |
| [`a11y/use-list`](./all.md#a11y-use-list) | [悪い例](./all.md#a11y-use-list-bad) · [良い例](./all.md#a11y-use-list-good) | 箇条書きに見えるテキストをリスト要素で表現します。 |
| [`vue/use-unique-element-ids`](./all.md#vue-use-unique-element-ids) | [悪い例](./all.md#vue-use-unique-element-ids-bad) · [良い例](./all.md#vue-use-unique-element-ids-good) | 静的 ID の代わりに useId() で再利用可能な ID を生成します。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md)
