---
title: アクセシビリティ ルール
---

# アクセシビリティ ルール

ルール名から、目的・重大度・適用範囲・設定・悪い例・良い例を確認できます。個別ページが現在の対応範囲を示す参照先です。

Vite+ では `lint.vize.rules` に設定し、`vp run lint` を実行します。型が必要なルールや、専用ファイル・追加設定が必要なルールは個別ページの前提を確認してください。

| ルール | 例 | 目的 |
| --- | --- | --- |
| [`a11y/alt-text`](./reference/a11y-alt-text.md) | [悪い例](./reference/a11y-alt-text.md#悪い) · [良い例](./reference/a11y-alt-text.md#良い) | 画像などのメディアに代替テキストを用意します。 |
| [`a11y/anchor-has-content`](./reference/a11y-anchor-has-content.md) | [悪い例](./reference/a11y-anchor-has-content.md#悪い) · [良い例](./reference/a11y-anchor-has-content.md#良い) | リンクに支援技術で読める内容を用意します。 |
| [`a11y/anchor-is-valid`](./reference/a11y-anchor-is-valid.md) | [悪い例](./reference/a11y-anchor-is-valid.md#悪い) · [良い例](./reference/a11y-anchor-is-valid.md#良い) | リンクの href に有効な移動先を指定します。 |
| [`a11y/aria-props`](./reference/a11y-aria-props.md) | [悪い例](./reference/a11y-aria-props.md#悪い) · [良い例](./reference/a11y-aria-props.md#良い) | 存在しない ARIA 属性を検出します。 |
| [`a11y/aria-role`](./reference/a11y-aria-role.md) | [悪い例](./reference/a11y-aria-role.md#悪い) · [良い例](./reference/a11y-aria-role.md#良い) | 有効で抽象的ではない ARIA role を指定します。 |
| [`a11y/aria-unsupported-elements`](./reference/a11y-aria-unsupported-elements.md) | [悪い例](./reference/a11y-aria-unsupported-elements.md#悪い) · [良い例](./reference/a11y-aria-unsupported-elements.md#良い) | ARIA 属性を使用できない要素への指定を検出します。 |
| [`a11y/click-events-have-key-events`](./reference/a11y-click-events-have-key-events.md) | [悪い例](./reference/a11y-click-events-have-key-events.md#悪い) · [良い例](./reference/a11y-click-events-have-key-events.md#良い) | クリックで操作する要素にキーボード操作も用意します。 |
| [`a11y/form-control-has-label`](./reference/a11y-form-control-has-label.md) | [悪い例](./reference/a11y-form-control-has-label.md#悪い) · [良い例](./reference/a11y-form-control-has-label.md#良い) | フォーム部品に関連付けられたラベルを用意します。 |
| [`a11y/heading-has-content`](./reference/a11y-heading-has-content.md) | [悪い例](./reference/a11y-heading-has-content.md#悪い) · [良い例](./reference/a11y-heading-has-content.md#良い) | 見出しに支援技術で読める内容を用意します。 |
| [`a11y/heading-levels`](./reference/a11y-heading-levels.md) | [悪い例](./reference/a11y-heading-levels.md#悪い) · [良い例](./reference/a11y-heading-levels.md#良い) | 見出しの階層を飛ばした指定を検出します。 |
| [`a11y/iframe-has-title`](./reference/a11y-iframe-has-title.md) | [悪い例](./reference/a11y-iframe-has-title.md#悪い) · [良い例](./reference/a11y-iframe-has-title.md#良い) | iframe に内容を説明する title を指定します。 |
| [`a11y/img-alt`](./reference/a11y-img-alt.md) | [悪い例](./reference/a11y-img-alt.md#悪い) · [良い例](./reference/a11y-img-alt.md#良い) | 画像に alt 属性を指定します。装飾画像は空の alt を使います。 |
| [`a11y/interactive-supports-focus`](./reference/a11y-interactive-supports-focus.md) | [悪い例](./reference/a11y-interactive-supports-focus.md#悪い) · [良い例](./reference/a11y-interactive-supports-focus.md#良い) | 操作可能な role の要素をフォーカス可能にします。 |
| [`a11y/label-has-for`](./reference/a11y-label-has-for.md) | [悪い例](./reference/a11y-label-has-for.md#悪い) · [良い例](./reference/a11y-label-has-for.md#良い) | label を対象のフォーム部品と関連付けます。 |
| [`a11y/landmark-roles`](./reference/a11y-landmark-roles.md) | [悪い例](./reference/a11y-landmark-roles.md#悪い) · [良い例](./reference/a11y-landmark-roles.md#良い) | ランドマーク role の配置と重複を検査します。 |
| [`a11y/media-has-caption`](./reference/a11y-media-has-caption.md) | [悪い例](./reference/a11y-media-has-caption.md#悪い) · [良い例](./reference/a11y-media-has-caption.md#良い) | 音声・動画に字幕を用意します。 |
| [`a11y/mouse-events-have-key-events`](./reference/a11y-mouse-events-have-key-events.md) | [悪い例](./reference/a11y-mouse-events-have-key-events.md#悪い) · [良い例](./reference/a11y-mouse-events-have-key-events.md#良い) | マウス操作と対応する focus / blur 操作を用意します。 |
| [`a11y/no-access-key`](./reference/a11y-no-access-key.md) | [悪い例](./reference/a11y-no-access-key.md#悪い) · [良い例](./reference/a11y-no-access-key.md#良い) | 環境のショートカットと衝突し得る accesskey を検出します。 |
| [`a11y/no-aria-hidden-on-focusable`](./reference/a11y-no-aria-hidden-on-focusable.md) | [悪い例](./reference/a11y-no-aria-hidden-on-focusable.md#悪い) · [良い例](./reference/a11y-no-aria-hidden-on-focusable.md#良い) | フォーカス可能な要素を aria-hidden で隠した指定を検出します。 |
| [`a11y/no-autofocus`](./reference/a11y-no-autofocus.md) | [悪い例](./reference/a11y-no-autofocus.md#悪い) · [良い例](./reference/a11y-no-autofocus.md#良い) | 意図せずフォーカスを移動させる autofocus を検出します。 |
| [`a11y/no-distracting-elements`](./reference/a11y-no-distracting-elements.md) | [悪い例](./reference/a11y-no-distracting-elements.md#悪い) · [良い例](./reference/a11y-no-distracting-elements.md#良い) | marquee や blink などの注意をそらす要素を検出します。 |
| [`a11y/no-i-for-icon`](./reference/a11y-no-i-for-icon.md) | [悪い例](./reference/a11y-no-i-for-icon.md#悪い) · [良い例](./reference/a11y-no-i-for-icon.md#良い) | アイコン用の i 要素を検出し、意味に合う要素を勧めます。 |
| [`a11y/no-redundant-roles`](./reference/a11y-no-redundant-roles.md) | [悪い例](./reference/a11y-no-redundant-roles.md#悪い) · [良い例](./reference/a11y-no-redundant-roles.md#良い) | 要素本来の意味と重複する ARIA role を検出します。 |
| [`a11y/no-refer-to-non-existent-id`](./reference/a11y-no-refer-to-non-existent-id.md) | [悪い例](./reference/a11y-no-refer-to-non-existent-id.md#悪い) · [良い例](./reference/a11y-no-refer-to-non-existent-id.md#良い) | 文書内に存在しない ID への参照を検出します。 |
| [`a11y/no-role-presentation-on-focusable`](./reference/a11y-no-role-presentation-on-focusable.md) | [悪い例](./reference/a11y-no-role-presentation-on-focusable.md#悪い) · [良い例](./reference/a11y-no-role-presentation-on-focusable.md#良い) | フォーカス可能な要素の意味を presentation で消した指定を検出します。 |
| [`a11y/no-static-element-interactions`](./reference/a11y-no-static-element-interactions.md) | [悪い例](./reference/a11y-no-static-element-interactions.md#悪い) · [良い例](./reference/a11y-no-static-element-interactions.md#良い) | 操作部品ではない要素へのイベント指定を検出します。 |
| [`a11y/placeholder-label-option`](./reference/a11y-placeholder-label-option.md) | [悪い例](./reference/a11y-placeholder-label-option.md#悪い) · [良い例](./reference/a11y-placeholder-label-option.md#良い) | select のプレースホルダー option に disabled または hidden を指定します。 |
| [`a11y/role-has-required-aria-props`](./reference/a11y-role-has-required-aria-props.md) | [悪い例](./reference/a11y-role-has-required-aria-props.md#悪い) · [良い例](./reference/a11y-role-has-required-aria-props.md#良い) | ARIA role が必要とする属性を指定します。 |
| [`a11y/tabindex-no-positive`](./reference/a11y-tabindex-no-positive.md) | [悪い例](./reference/a11y-tabindex-no-positive.md#悪い) · [良い例](./reference/a11y-tabindex-no-positive.md#良い) | 通常のフォーカス順序を変える正の tabindex を検出します。 |
| [`a11y/use-list`](./reference/a11y-use-list.md) | [悪い例](./reference/a11y-use-list.md#悪い) · [良い例](./reference/a11y-use-list.md#良い) | 箇条書きに見えるテキストをリスト要素で表現します。 |
| [`vue/use-unique-element-ids`](./reference/vue-use-unique-element-ids.md) | [悪い例](./reference/vue-use-unique-element-ids.md#悪い) · [良い例](./reference/vue-use-unique-element-ids.md#良い) | 静的 ID の代わりに useId() で再利用可能な ID を生成します。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md)
