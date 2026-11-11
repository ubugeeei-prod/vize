---
title: "アクセシビリティのルール"
description: "ARIA、キーボード操作、ラベル、ランドマーク、メディアの代替手段を検査し、支援技術で使えるテンプレートにします。"
---

# アクセシビリティのルール

ARIA、キーボード操作、ラベル、ランドマーク、メディアの代替手段を検査し、支援技術で使えるテンプレートにします。

各ルールのページに、診断が出る例・出ない例と、そのルールを有効にする設定があります。

## 既定 (25)

既定のプリセット `happy-path` で追加されるルールです。

| ルール | 検出する内容 | 自動修正 |
| --- | --- | --- |
| <span id="a11y-alt-text"></span>[`a11y/alt-text`](./reference/a11y-alt-text.md) | 画像などのメディアに代替テキストを用意します。 |  |
| <span id="a11y-anchor-has-content"></span>[`a11y/anchor-has-content`](./reference/a11y-anchor-has-content.md) | リンクに支援技術で読める内容を用意します。 |  |
| <span id="a11y-anchor-is-valid"></span>[`a11y/anchor-is-valid`](./reference/a11y-anchor-is-valid.md) | リンクの href に有効な移動先を指定します。 |  |
| <span id="a11y-aria-props"></span>[`a11y/aria-props`](./reference/a11y-aria-props.md) | 存在しない ARIA 属性を検出します。 |  |
| <span id="a11y-aria-role"></span>[`a11y/aria-role`](./reference/a11y-aria-role.md) | 抽象ロールを除く、有効な ARIA role を指定します。 |  |
| <span id="a11y-aria-unsupported-elements"></span>[`a11y/aria-unsupported-elements`](./reference/a11y-aria-unsupported-elements.md) | ARIA 属性を使用できない要素への指定を検出します。 |  |
| <span id="a11y-click-events-have-key-events"></span>[`a11y/click-events-have-key-events`](./reference/a11y-click-events-have-key-events.md) | クリックで操作する要素にキーボード操作も用意します。 |  |
| <span id="a11y-form-control-has-label"></span>[`a11y/form-control-has-label`](./reference/a11y-form-control-has-label.md) | フォーム部品に関連付けられたラベルを用意します。 |  |
| <span id="a11y-heading-has-content"></span>[`a11y/heading-has-content`](./reference/a11y-heading-has-content.md) | 見出しに支援技術で読める内容を用意します。 |  |
| <span id="a11y-iframe-has-title"></span>[`a11y/iframe-has-title`](./reference/a11y-iframe-has-title.md) | iframe に内容を説明する title を指定します。 |  |
| <span id="a11y-interactive-supports-focus"></span>[`a11y/interactive-supports-focus`](./reference/a11y-interactive-supports-focus.md) | 操作可能な role の要素をフォーカス可能にします。 |  |
| <span id="a11y-label-has-for"></span>[`a11y/label-has-for`](./reference/a11y-label-has-for.md) | label を対象のフォーム部品と関連付けます。 |  |
| <span id="a11y-media-has-caption"></span>[`a11y/media-has-caption`](./reference/a11y-media-has-caption.md) | 音声・動画に字幕を用意します。 |  |
| <span id="a11y-mouse-events-have-key-events"></span>[`a11y/mouse-events-have-key-events`](./reference/a11y-mouse-events-have-key-events.md) | マウス操作と対応する focus / blur 操作を用意します。 |  |
| <span id="a11y-no-access-key"></span>[`a11y/no-access-key`](./reference/a11y-no-access-key.md) | 環境のショートカットと衝突し得る accesskey を検出します。 |  |
| <span id="a11y-no-aria-hidden-on-focusable"></span>[`a11y/no-aria-hidden-on-focusable`](./reference/a11y-no-aria-hidden-on-focusable.md) | フォーカス可能な要素を aria-hidden で支援技術から隠す指定を検出します。 |  |
| <span id="a11y-no-autofocus"></span>[`a11y/no-autofocus`](./reference/a11y-no-autofocus.md) | 意図せずフォーカスを移動させる autofocus を検出します。 |  |
| <span id="a11y-no-distracting-elements"></span>[`a11y/no-distracting-elements`](./reference/a11y-no-distracting-elements.md) | marquee や blink などの注意をそらす要素を検出します。 |  |
| <span id="a11y-no-i-for-icon"></span>[`a11y/no-i-for-icon`](./reference/a11y-no-i-for-icon.md) | アイコン用の i 要素を検出し、意味に合う要素を勧めます。 |  |
| <span id="a11y-no-redundant-roles"></span>[`a11y/no-redundant-roles`](./reference/a11y-no-redundant-roles.md) | 要素本来の意味と重複する ARIA role を検出します。 | ✓ |
| <span id="a11y-no-refer-to-non-existent-id"></span>[`a11y/no-refer-to-non-existent-id`](./reference/a11y-no-refer-to-non-existent-id.md) | 文書内に存在しない ID への参照を検出します。 |  |
| <span id="a11y-no-role-presentation-on-focusable"></span>[`a11y/no-role-presentation-on-focusable`](./reference/a11y-no-role-presentation-on-focusable.md) | フォーカス可能な要素への role="presentation" または role="none" の指定を検出します。 |  |
| <span id="a11y-no-static-element-interactions"></span>[`a11y/no-static-element-interactions`](./reference/a11y-no-static-element-interactions.md) | 操作部品ではない要素へのイベント指定を検出します。 |  |
| <span id="a11y-role-has-required-aria-props"></span>[`a11y/role-has-required-aria-props`](./reference/a11y-role-has-required-aria-props.md) | ARIA role が必要とする属性を指定します。 |  |
| <span id="a11y-tabindex-no-positive"></span>[`a11y/tabindex-no-positive`](./reference/a11y-tabindex-no-positive.md) | 通常のフォーカス順序を変える正の tabindex を検出します。 |  |

## より厳しいプリセット (5)

`opinionated`・`nuxt`・`ecosystem` のいずれかにだけ含まれます。どれに含まれるかはプリセット列を参照してください。

| ルール | 検出する内容 | プリセット | 自動修正 |
| --- | --- | --- | --- |
| <span id="a11y-heading-levels"></span>[`a11y/heading-levels`](./reference/a11y-heading-levels.md) | 見出しの階層を飛ばした指定を検出します。 | `nuxt`, `opinionated` |  |
| <span id="a11y-landmark-roles"></span>[`a11y/landmark-roles`](./reference/a11y-landmark-roles.md) | ランドマーク role の配置と重複を検査します。 | `nuxt`, `opinionated` |  |
| <span id="a11y-placeholder-label-option"></span>[`a11y/placeholder-label-option`](./reference/a11y-placeholder-label-option.md) | required の select を値が空のプレースホルダー option から始めます。 | `nuxt`, `opinionated` |  |
| <span id="a11y-use-list"></span>[`a11y/use-list`](./reference/a11y-use-list.md) | 箇条書きに見えるテキストをリスト要素で表現します。 | `nuxt`, `opinionated` |  |
| <span id="vue-use-unique-element-ids"></span>[`vue/use-unique-element-ids`](./reference/vue-use-unique-element-ids.md) | 静的 ID の代わりに useId() で一意な ID を生成します。 | `nuxt`, `opinionated` |  |

## 個別に有効化 (2)

どのプリセットにも含まれません。`withVue` に渡す `linter.rules` でルール名を指定して有効にします。

| ルール | 検出する内容 | 自動修正 |
| --- | --- | --- |
| <span id="a11y-img-alt"></span>[`a11y/img-alt`](./reference/a11y-img-alt.md) | 画像に alt 属性を指定します。装飾画像は空の alt を使います。 |  |
| <span id="a11y-role-img-has-name"></span>[`a11y/role-img-has-name`](./reference/a11y-role-img-has-name.md) | role="img" の要素にアクセシブルな名前を要求します。 |  |

[ルールの概要](../rules/index.md) · [ルールのオプション](./options.md) · [ESLint からの移行](./migration.md)
