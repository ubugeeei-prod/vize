---
title: "アクセシビリティ ルール"
---

# アクセシビリティ ルール

このページに、各ルールの目的・前提・設定・完全な悪い例と良い例をまとめています。変更箇所は行の色で示し、コピーしたコードには完全なソースを保持します。各例に現在の対応範囲も明記しています。


| ルール | 例 | 目的 |
| --- | --- | --- |
| [`a11y/alt-text`](#a11y-alt-text) | [悪い](#a11y-alt-text-bad) · [良い](#a11y-alt-text-good) | 画像などのメディアに代替テキストを用意します。 |
| [`a11y/anchor-has-content`](#a11y-anchor-has-content) | [悪い](#a11y-anchor-has-content-bad) · [良い](#a11y-anchor-has-content-good) | リンクに支援技術で読める内容を用意します。 |
| [`a11y/anchor-is-valid`](#a11y-anchor-is-valid) | [悪い](#a11y-anchor-is-valid-bad) · [良い](#a11y-anchor-is-valid-good) | リンクの href に有効な移動先を指定します。 |
| [`a11y/aria-props`](#a11y-aria-props) | [悪い](#a11y-aria-props-bad) · [良い](#a11y-aria-props-good) | 存在しない ARIA 属性を検出します。 |
| [`a11y/aria-role`](#a11y-aria-role) | [悪い](#a11y-aria-role-bad) · [良い](#a11y-aria-role-good) | 有効で抽象的ではない ARIA role を指定します。 |
| [`a11y/aria-unsupported-elements`](#a11y-aria-unsupported-elements) | [悪い](#a11y-aria-unsupported-elements-bad) · [良い](#a11y-aria-unsupported-elements-good) | ARIA 属性を使用できない要素への指定を検出します。 |
| [`a11y/click-events-have-key-events`](#a11y-click-events-have-key-events) | [悪い](#a11y-click-events-have-key-events-bad) · [良い](#a11y-click-events-have-key-events-good) | クリックで操作する要素にキーボード操作も用意します。 |
| [`a11y/form-control-has-label`](#a11y-form-control-has-label) | [悪い](#a11y-form-control-has-label-bad) · [良い](#a11y-form-control-has-label-good) | フォーム部品に関連付けられたラベルを用意します。 |
| [`a11y/heading-has-content`](#a11y-heading-has-content) | [悪い](#a11y-heading-has-content-bad) · [良い](#a11y-heading-has-content-good) | 見出しに支援技術で読める内容を用意します。 |
| [`a11y/heading-levels`](#a11y-heading-levels) | [悪い](#a11y-heading-levels-bad) · [良い](#a11y-heading-levels-good) | 見出しの階層を飛ばした指定を検出します。 |
| [`a11y/iframe-has-title`](#a11y-iframe-has-title) | [悪い](#a11y-iframe-has-title-bad) · [良い](#a11y-iframe-has-title-good) | iframe に内容を説明する title を指定します。 |
| [`a11y/img-alt`](#a11y-img-alt) | [悪い](#a11y-img-alt-bad) · [良い](#a11y-img-alt-good) | 画像に alt 属性を指定します。装飾画像は空の alt を使います。 |
| [`a11y/interactive-supports-focus`](#a11y-interactive-supports-focus) | [悪い](#a11y-interactive-supports-focus-bad) · [良い](#a11y-interactive-supports-focus-good) | 操作可能な role の要素をフォーカス可能にします。 |
| [`a11y/label-has-for`](#a11y-label-has-for) | [悪い](#a11y-label-has-for-bad) · [良い](#a11y-label-has-for-good) | label を対象のフォーム部品と関連付けます。 |
| [`a11y/landmark-roles`](#a11y-landmark-roles) | [悪い](#a11y-landmark-roles-bad) · [良い](#a11y-landmark-roles-good) | ランドマーク role の配置と重複を検査します。 |
| [`a11y/media-has-caption`](#a11y-media-has-caption) | [悪い](#a11y-media-has-caption-bad) · [良い](#a11y-media-has-caption-good) | 音声・動画に字幕を用意します。 |
| [`a11y/mouse-events-have-key-events`](#a11y-mouse-events-have-key-events) | [悪い](#a11y-mouse-events-have-key-events-bad) · [良い](#a11y-mouse-events-have-key-events-good) | マウス操作と対応する focus / blur 操作を用意します。 |
| [`a11y/no-access-key`](#a11y-no-access-key) | [悪い](#a11y-no-access-key-bad) · [良い](#a11y-no-access-key-good) | 環境のショートカットと衝突し得る accesskey を検出します。 |
| [`a11y/no-aria-hidden-on-focusable`](#a11y-no-aria-hidden-on-focusable) | [悪い](#a11y-no-aria-hidden-on-focusable-bad) · [良い](#a11y-no-aria-hidden-on-focusable-good) | フォーカス可能な要素を aria-hidden で隠した指定を検出します。 |
| [`a11y/no-autofocus`](#a11y-no-autofocus) | [悪い](#a11y-no-autofocus-bad) · [良い](#a11y-no-autofocus-good) | 意図せずフォーカスを移動させる autofocus を検出します。 |
| [`a11y/no-distracting-elements`](#a11y-no-distracting-elements) | [悪い](#a11y-no-distracting-elements-bad) · [良い](#a11y-no-distracting-elements-good) | marquee や blink などの注意をそらす要素を検出します。 |
| [`a11y/no-i-for-icon`](#a11y-no-i-for-icon) | [悪い](#a11y-no-i-for-icon-bad) · [良い](#a11y-no-i-for-icon-good) | アイコン用の i 要素を検出し、意味に合う要素を勧めます。 |
| [`a11y/no-redundant-roles`](#a11y-no-redundant-roles) | [悪い](#a11y-no-redundant-roles-bad) · [良い](#a11y-no-redundant-roles-good) | 要素本来の意味と重複する ARIA role を検出します。 |
| [`a11y/no-refer-to-non-existent-id`](#a11y-no-refer-to-non-existent-id) | [悪い](#a11y-no-refer-to-non-existent-id-bad) · [良い](#a11y-no-refer-to-non-existent-id-good) | 文書内に存在しない ID への参照を検出します。 |
| [`a11y/no-role-presentation-on-focusable`](#a11y-no-role-presentation-on-focusable) | [悪い](#a11y-no-role-presentation-on-focusable-bad) · [良い](#a11y-no-role-presentation-on-focusable-good) | フォーカス可能な要素の意味を presentation で消した指定を検出します。 |
| [`a11y/no-static-element-interactions`](#a11y-no-static-element-interactions) | [悪い](#a11y-no-static-element-interactions-bad) · [良い](#a11y-no-static-element-interactions-good) | 操作部品ではない要素へのイベント指定を検出します。 |
| [`a11y/placeholder-label-option`](#a11y-placeholder-label-option) | [悪い](#a11y-placeholder-label-option-bad) · [良い](#a11y-placeholder-label-option-good) | select のプレースホルダー option に disabled または hidden を指定します。 |
| [`a11y/role-has-required-aria-props`](#a11y-role-has-required-aria-props) | [悪い](#a11y-role-has-required-aria-props-bad) · [良い](#a11y-role-has-required-aria-props-good) | ARIA role が必要とする属性を指定します。 |
| [`a11y/tabindex-no-positive`](#a11y-tabindex-no-positive) | [悪い](#a11y-tabindex-no-positive-bad) · [良い](#a11y-tabindex-no-positive-good) | 通常のフォーカス順序を変える正の tabindex を検出します。 |
| [`a11y/use-list`](#a11y-use-list) | [悪い](#a11y-use-list-bad) · [良い](#a11y-use-list-good) | 箇条書きに見えるテキストをリスト要素で表現します。 |
| [`vue/use-unique-element-ids`](#vue-use-unique-element-ids) | [悪い](#vue-use-unique-element-ids-bad) · [良い](#vue-use-unique-element-ids-good) | 静的 ID の代わりに useId() で再利用可能な ID を生成します。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)

### `a11y/alt-text`

画像などのメディアに代替テキストを用意します。

[悪い例](#a11y-alt-text-bad) · [良い例](#a11y-alt-text-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/alt-text": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-alt-text-bad"></span>

**悪い**

画像の送信ボタンには URL しかなく、操作を説明する `alt` がありません。

```vue annotate="remove:2"
<template>
  <input type="image" src="/submit.png" />
</template>
```

<span id="a11y-alt-text-good"></span>

**良い**

`alt="Submit search"` を追加し、検索を送信する操作の名前を指定します。

```vue annotate="add:2"
<template>
  <input type="image" src="/submit.png" alt="Submit search" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/alt_text.rs#L33) · [全ルール](all.md)

### `a11y/anchor-has-content`

リンクに支援技術で読める内容を用意します。

[悪い例](#a11y-anchor-has-content-bad) · [良い例](#a11y-anchor-has-content-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/anchor-has-content": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-anchor-has-content-bad"></span>

**悪い**

`/settings` へのリンクが空で、移動先を説明する内容がありません。

```vue annotate="remove:2"
<template>
  <a href="/settings"></a>
</template>
```

<span id="a11y-anchor-has-content-good"></span>

**良い**

同じリンクに `Settings` の文字を入れ、移動先を示します。

```vue annotate="add:2"
<template>
  <a href="/settings">Settings</a>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_has_content.rs#L16) · [全ルール](all.md)

### `a11y/anchor-is-valid`

リンクの href に有効な移動先を指定します。

[悪い例](#a11y-anchor-is-valid-bad) · [良い例](#a11y-anchor-is-valid-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/anchor-is-valid": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-anchor-is-valid-bad"></span>

**悪い**

一つ目は操作に `#` を使い、二つ目は JavaScript URL を使っています。どちらも通常の移動先を持つリンクではありません。

```vue annotate="remove:2,3"
<template>
  <a href="#" @click="openPanel">Open panel</a>
  <a href="JaVaScRiPt:void(0)">Run action</a>
</template>
```

<span id="a11y-anchor-is-valid-good"></span>

**良い**

`openPanel` は button で実行し、リンクには実際の移動先 `/docs/javascript-urls` を指定します。

```vue annotate="add:2,3"
<template>
  <button type="button" @click="openPanel">Open panel</button>
  <a href="/docs/javascript-urls">JavaScript URL guide</a>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_is_valid.rs#L30) · [全ルール](all.md)

### `a11y/aria-props`

存在しない ARIA 属性を検出します。

[悪い例](#a11y-aria-props-bad) · [良い例](#a11y-aria-props-good)

既定の重大度: `error`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/aria-props": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-aria-props-bad"></span>

**悪い**

`aria-lable` は綴りが誤っており、対応する ARIA 属性ではありません。

```vue annotate="remove:2"
<template>
  <button aria-lable="Save changes">Save</button>
</template>
```

<span id="a11y-aria-props-good"></span>

**良い**

正しい `aria-label` に変更してボタンの名前を指定します。

```vue annotate="add:2"
<template>
  <button aria-label="Save changes">Save</button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_props.rs#L18) · [全ルール](all.md)

### `a11y/aria-role`

有効で抽象的ではない ARIA role を指定します。

[悪い例](#a11y-aria-role-bad) · [良い例](#a11y-aria-role-good)

既定の重大度: `error`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/aria-role": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-aria-role-bad"></span>

**悪い**

`datepicker` は認識される ARIA role ではありません。

```vue annotate="remove:2"
<template>
  <section role="datepicker">...</section>
</template>
```

<span id="a11y-aria-role-good"></span>

**良い**

認識される `dialog` を使い、日付を選択する領域の名前も指定します。

```vue annotate="add:2"
<template>
  <section role="dialog" aria-label="Choose a date">...</section>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_role.rs#L21) · [全ルール](all.md)

### `a11y/aria-unsupported-elements`

ARIA 属性を使用できない要素への指定を検出します。

[悪い例](#a11y-aria-unsupported-elements-bad) · [良い例](#a11y-aria-unsupported-elements-good)

既定の重大度: `error`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/aria-unsupported-elements": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-aria-unsupported-elements-bad"></span>

**悪い**

ARIA 属性を使えない `meta` に `aria-hidden` を指定しています。

```vue annotate="remove:2"
<template>
  <meta charset="utf-8" aria-hidden="true" />
</template>
```

<span id="a11y-aria-unsupported-elements-good"></span>

**良い**

ARIA 属性だけを取り除き、文字コードの宣言は維持します。

```vue annotate="add:2"
<template>
  <meta charset="utf-8" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_unsupported_elements.rs#L18) · [全ルール](all.md)

### `a11y/click-events-have-key-events`

クリックで操作する要素にキーボード操作も用意します。

[悪い例](#a11y-click-events-have-key-events-bad) · [良い例](#a11y-click-events-have-key-events-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

対話的な役割を持たない通常要素が対象です。button や対話的な ARIA role を持つ要素はこの検出の対象外です。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/click-events-have-key-events": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-click-events-have-key-events-bad"></span>

**悪い**

通常の `div` に click handler だけを指定し、キーボード操作に対応していません。

```vue annotate="remove:2"
<template>
<div @click="activate">Activate</div>
</template>
```

<span id="a11y-click-events-have-key-events-good"></span>

**良い**

同じ `activate` を button に指定し、標準のキーボード操作を使います。

```vue annotate="add:2"
<template>
<button @click="activate">Activate</button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/click_events_have_key_events.rs#L17) · [全ルール](all.md)

### `a11y/form-control-has-label`

フォーム部品に関連付けられたラベルを用意します。

[悪い例](#a11y-form-control-has-label-bad) · [良い例](#a11y-form-control-has-label-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/form-control-has-label": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-form-control-has-label-bad"></span>

**悪い**

検索欄に入力内容を説明する label がありません。

```vue annotate="remove:2"
<template>
  <input type="search" />
</template>
```

<span id="a11y-form-control-has-label-good"></span>

**良い**

入力欄を label で囲み、`Search` の文字と関連付けます。

```vue annotate="add:2,3,4,5"
<template>
  <label>
    Search
    <input type="search" />
  </label>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/form_control_has_label.rs#L19) · [全ルール](all.md)

### `a11y/heading-has-content`

見出しに支援技術で読める内容を用意します。

[悪い例](#a11y-heading-has-content-bad) · [良い例](#a11y-heading-has-content-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/heading-has-content": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-heading-has-content-bad"></span>

**悪い**

`h2` の見出しレベルはありますが、見出しの内容が空です。

```vue annotate="remove:2"
<template>
  <h2></h2>
</template>
```

<span id="a11y-heading-has-content-good"></span>

**良い**

同じ `h2` に `Billing settings` の内容を入れます。

```vue annotate="add:2"
<template>
  <h2>Billing settings</h2>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/heading_has_content.rs#L17) · [全ルール](all.md)

### `a11y/heading-levels`

見出しの階層を飛ばした指定を検出します。

[悪い例](#a11y-heading-levels-bad) · [良い例](#a11y-heading-levels-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/heading-levels": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-heading-levels-bad"></span>

**悪い**

見出しが `h1` から `h3` に飛び、レベル 2 を省略しています。

```vue annotate="remove:3"
<template>
  <h1>Account</h1>
  <h3>Billing</h3>
</template>
```

<span id="a11y-heading-levels-good"></span>

**良い**

請求設定の見出しを `h2` にし、階層を順に並べます。

```vue annotate="add:3"
<template>
  <h1>Account</h1>
  <h2>Billing</h2>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/heading_levels.rs#L34) · [全ルール](all.md)

### `a11y/iframe-has-title`

iframe に内容を説明する title を指定します。

[悪い例](#a11y-iframe-has-title-bad) · [良い例](#a11y-iframe-has-title-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/iframe-has-title": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-iframe-has-title-bad"></span>

**悪い**

決済画面の iframe に URL はありますが、内容を説明する title がありません。

```vue annotate="remove:2"
<template>
  <iframe src="/checkout"></iframe>
</template>
```

<span id="a11y-iframe-has-title-good"></span>

**良い**

`title="Checkout preview"` で iframe の内容を説明します。

```vue annotate="add:2"
<template>
  <iframe src="/checkout" title="Checkout preview"></iframe>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/iframe_has_title.rs#L15) · [全ルール](all.md)

### `a11y/img-alt`

画像に alt 属性を指定します。装飾画像は空の alt を使います。

[悪い例](#a11y-img-alt-bad) · [良い例](#a11y-img-alt-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/img-alt": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-img-alt-bad"></span>

**悪い**

アバター画像に `alt` 属性がなく、画像の代替テキストを確認できません。

```vue annotate="remove:2"
<template>
  <img src="/avatar.png" />
</template>
```

<span id="a11y-img-alt-good"></span>

**良い**

`alt="User avatar"` で画像の代わりとなる文字を指定します。

```vue annotate="add:2"
<template>
  <img src="/avatar.png" alt="User avatar" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/img_alt.rs#L16) · [全ルール](all.md)

### `a11y/interactive-supports-focus`

操作可能な role の要素をフォーカス可能にします。

[悪い例](#a11y-interactive-supports-focus-bad) · [良い例](#a11y-interactive-supports-focus-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/interactive-supports-focus": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-interactive-supports-focus-bad"></span>

**悪い**

span に button の role と click handler を付けても、キーボードでフォーカスできる要素にはなりません。

```vue annotate="remove:2"
<template>
  <span role="button" @click="open">Open</span>
</template>
```

<span id="a11y-interactive-supports-focus-good"></span>

**良い**

フォーカスできる標準の button に変更し、同じ `open` を実行します。

```vue annotate="add:2"
<template>
  <button type="button" @click="open">Open</button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/interactive_supports_focus.rs#L31) · [全ルール](all.md)

### `a11y/label-has-for`

label を対象のフォーム部品と関連付けます。

[悪い例](#a11y-label-has-for-bad) · [良い例](#a11y-label-has-for-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/label-has-for": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-label-has-for-bad"></span>

**悪い**

離れた label に for がなく、入力欄を囲んでもいないため関連付けがありません。

```vue annotate="remove:2"
<template>
  <label>Email</label>
  <input id="email" />
</template>
```

<span id="a11y-label-has-for-good"></span>

**良い**

`for="email"` を入力欄の ID と一致させて関連付けます。

```vue annotate="add:2"
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/label_has_for.rs#L27) · [全ルール](all.md)

### `a11y/landmark-roles`

ランドマーク role の配置と重複を検査します。

[悪い例](#a11y-landmark-roles-bad) · [良い例](#a11y-landmark-roles-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/landmark-roles": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-landmark-roles-bad"></span>

**悪い**

同じテンプレートに main が二つあり、主要な領域が重複しています。

```vue annotate="remove:3"
<template>
  <main>Dashboard</main>
  <main>Settings</main>
</template>
```

<span id="a11y-landmark-roles-good"></span>

**良い**

Dashboard を main として残し、Settings を名前付きの nav に変更します。

```vue annotate="add:3"
<template>
  <main>Dashboard</main>
  <nav aria-label="Settings">...</nav>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/landmark_roles.rs#L44) · [全ルール](all.md)

### `a11y/media-has-caption`

音声・動画に字幕を用意します。

[悪い例](#a11y-media-has-caption-bad) · [良い例](#a11y-media-has-caption-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/media-has-caption": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-media-has-caption-bad"></span>

**悪い**

動画に再生操作はありますが、字幕の track がありません。

```vue annotate="remove:2"
<template>
  <video src="/demo.mp4" controls />
</template>
```

<span id="a11y-media-has-caption-good"></span>

**良い**

同じ動画に `kind="captions"` の track を追加して英語の字幕を指定します。

```vue annotate="add:2,3,4"
<template>
  <video src="/demo.mp4" controls>
    <track kind="captions" src="/demo.en.vtt" srclang="en" label="English" />
  </video>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/media_has_caption.rs#L30) · [全ルール](all.md)

### `a11y/mouse-events-have-key-events`

マウス操作と対応する focus / blur 操作を用意します。

[悪い例](#a11y-mouse-events-have-key-events-bad) · [良い例](#a11y-mouse-events-have-key-events-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/mouse-events-have-key-events": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-mouse-events-have-key-events-bad"></span>

**悪い**

プレビューの表示切り替えを mouseenter と mouseleave だけに指定しています。

```vue annotate="remove:2"
<template>
  <div @mouseenter="showPreview" @mouseleave="hidePreview">Preview</div>
</template>
```

<span id="a11y-mouse-events-have-key-events-good"></span>

**良い**

フォーカスできる button で、同じ操作を focus と blur からも実行します。

```vue annotate="add:2,3,4,5,6,7,8,9,10"
<template>
  <button
    type="button"
    @focus="showPreview"
    @blur="hidePreview"
    @mouseenter="showPreview"
    @mouseleave="hidePreview"
  >
    Preview
  </button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/mouse_events_have_key_events.rs#L30) · [全ルール](all.md)

### `a11y/no-access-key`

環境のショートカットと衝突し得る accesskey を検出します。

[悪い例](#a11y-no-access-key-bad) · [良い例](#a11y-no-access-key-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/no-access-key": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-access-key-bad"></span>

**悪い**

`accesskey="s"` がブラウザーや支援技術のショートカットと競合する可能性があります。

```vue annotate="remove:2"
<template>
  <button accesskey="s">Save</button>
</template>
```

<span id="a11y-no-access-key-good"></span>

**良い**

accesskey を取り除き、通常の Save ボタンは残します。

```vue annotate="add:2"
<template>
  <button>Save</button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_access_key.rs#L19) · [全ルール](all.md)

### `a11y/no-aria-hidden-on-focusable`

フォーカス可能な要素を aria-hidden で隠した指定を検出します。

[悪い例](#a11y-no-aria-hidden-on-focusable-bad) · [良い例](#a11y-no-aria-hidden-on-focusable-good)

既定の重大度: `error`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/no-aria-hidden-on-focusable": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-aria-hidden-on-focusable-bad"></span>

**悪い**

フォーカスできる Close ボタンを `aria-hidden="true"` でアクセシビリティ ツリーから隠しています。

```vue annotate="remove:2"
<template>
  <button aria-hidden="true" @click="close">Close</button>
</template>
```

<span id="a11y-no-aria-hidden-on-focusable-good"></span>

**良い**

ボタンを隠さず、Close の aria-label を指定します。

```vue annotate="add:2"
<template>
  <button aria-label="Close" @click="close">Close</button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_aria_hidden_on_focusable.rs#L19) · [全ルール](all.md)

### `a11y/no-autofocus`

意図せずフォーカスを移動させる autofocus を検出します。

[悪い例](#a11y-no-autofocus-bad) · [良い例](#a11y-no-autofocus-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/no-autofocus": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-autofocus-bad"></span>

**悪い**

検索の入力欄に autofocus があり、表示時に利用者の操作なしでフォーカスを要求します。

```vue annotate="remove:2"
<template>
  <input autofocus name="query" />
</template>
```

<span id="a11y-no-autofocus-good"></span>

**良い**

autofocus を取り除き、検索欄はそのまま残します。

```vue annotate="add:2"
<template>
  <input name="query" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_autofocus.rs#L19) · [全ルール](all.md)

### `a11y/no-distracting-elements`

marquee や blink などの注意をそらす要素を検出します。

[悪い例](#a11y-no-distracting-elements-bad) · [良い例](#a11y-no-distracting-elements-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/no-distracting-elements": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-distracting-elements-bad"></span>

**悪い**

marquee を使って文字を自動的に動かしています。

```vue annotate="remove:2"
<template>
  <marquee>Limited offer</marquee>
</template>
```

<span id="a11y-no-distracting-elements-good"></span>

**良い**

同じ案内を p に入れ、自動的に動く要素を使いません。

```vue annotate="add:2"
<template>
  <p>Limited offer</p>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_distracting_elements.rs#L16) · [全ルール](all.md)

### `a11y/no-i-for-icon`

アイコン用の i 要素を検出し、意味に合う要素を勧めます。

[悪い例](#a11y-no-i-for-icon-bad) · [良い例](#a11y-no-i-for-icon-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/no-i-for-icon": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-i-for-icon-bad"></span>

**悪い**

アイコンを i で表示しており、アイコンだけの操作を説明する文字がありません。

```vue annotate="remove:3"
<template>
  <button>
    <i class="material-icons">delete</i>
  </button>
</template>
```

<span id="a11y-no-i-for-icon-good"></span>

**良い**

装飾の span でアイコンを隠し、別の `Delete item` の文字で操作を説明します。

```vue annotate="add:3,4"
<template>
  <button>
    <span class="material-icons" aria-hidden="true">delete</span>
    <span class="sr-only">Delete item</span>
  </button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_i_for_icon.rs#L35) · [全ルール](all.md)

### `a11y/no-redundant-roles`

要素本来の意味と重複する ARIA role を検出します。

[悪い例](#a11y-no-redundant-roles-bad) · [良い例](#a11y-no-redundant-roles-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: 対応する検出で利用可能  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/no-redundant-roles": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-redundant-roles-bad"></span>

**悪い**

button は元から button の役割を持つため、同じ role を重複して指定しています。

```vue annotate="remove:2"
<template>
  <button role="button">Save</button>
</template>
```

<span id="a11y-no-redundant-roles-good"></span>

**良い**

重複する role を取り除き、HTML の標準の役割を使います。

```vue annotate="add:2"
<template>
  <button>Save</button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_redundant_roles/report.rs#L31) · [全ルール](all.md)

### `a11y/no-refer-to-non-existent-id`

文書内に存在しない ID への参照を検出します。

[悪い例](#a11y-no-refer-to-non-existent-id-bad) · [良い例](#a11y-no-refer-to-non-existent-id-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/no-refer-to-non-existent-id": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-refer-to-non-existent-id-bad"></span>

**悪い**

aria-labelledby が save-label を参照していますが、その ID の要素がありません。

```vue
<template>
  <button aria-labelledby="save-label">Save</button>
</template>
```

<span id="a11y-no-refer-to-non-existent-id-good"></span>

**良い**

一致する ID の span を追加し、ボタンの名前を参照できるようにします。

```vue annotate="add:2"
<template>
  <span id="save-label">Save changes</span>
  <button aria-labelledby="save-label">Save</button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_refer_to_non_existent_id.rs#L37) · [全ルール](all.md)

### `a11y/no-role-presentation-on-focusable`

フォーカス可能な要素の意味を presentation で消した指定を検出します。

[悪い例](#a11y-no-role-presentation-on-focusable-bad) · [良い例](#a11y-no-role-presentation-on-focusable-good)

既定の重大度: `error`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/no-role-presentation-on-focusable": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-role-presentation-on-focusable-bad"></span>

**悪い**

focus できる Billing のリンクに role=presentation を指定し、操作可能なリンクの役割と矛盾させています。ブラウザはこの presentation の指定を無視する必要があります。

```vue annotate="remove:2"
<template>
  <a href="/billing" role="presentation">Billing</a>
</template>
```

<span id="a11y-no-role-presentation-on-focusable-good"></span>

**良い**

矛盾する presentation の指定を除き、Billing への標準のリンクの役割を使います。

```vue annotate="add:2"
<template>
  <a href="/billing">Billing</a>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_role_presentation_on_focusable.rs#L19) · [全ルール](all.md)

### `a11y/no-static-element-interactions`

操作部品ではない要素へのイベント指定を検出します。

[悪い例](#a11y-no-static-element-interactions-bad) · [良い例](#a11y-no-static-element-interactions-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/no-static-element-interactions": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-static-element-interactions-bad"></span>

**悪い**

操作の役割がない section に Enter キーの操作を指定しています。

```vue annotate="remove:2"
<template>
  <section @keydown.enter="select">Select</section>
</template>
```

<span id="a11y-no-static-element-interactions-good"></span>

**良い**

同じ Enter キーの操作を標準の button に指定し、要素自体に操作の意味を持たせます。

```vue annotate="add:2"
<template>
  <button type="button" @keydown.enter="select">Select</button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_static_element_interactions.rs#L31) · [全ルール](all.md)

### `a11y/placeholder-label-option`

select のプレースホルダー option に disabled または hidden を指定します。

[悪い例](#a11y-placeholder-label-option-bad) · [良い例](#a11y-placeholder-label-option-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/placeholder-label-option": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-placeholder-label-option-bad"></span>

**悪い**

空の値を持つ案内の option が、国の選択肢と同じように選択できる状態です。

```vue annotate="remove:3"
<template>
  <select v-model="country">
    <option value="">Choose a country</option>
    <option value="jp">Japan</option>
  </select>
</template>
```

<span id="a11y-placeholder-label-option-good"></span>

**良い**

disabled を追加し、案内を Japan の選択肢と区別します。

```vue annotate="add:3"
<template>
  <select v-model="country">
    <option value="" disabled>Choose a country</option>
    <option value="jp">Japan</option>
  </select>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/placeholder_label_option.rs#L36) · [全ルール](all.md)

### `a11y/role-has-required-aria-props`

ARIA role が必要とする属性を指定します。

[悪い例](#a11y-role-has-required-aria-props-bad) · [良い例](#a11y-role-has-required-aria-props-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/role-has-required-aria-props": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-role-has-required-aria-props-bad"></span>

**悪い**

checkbox の role に、選択状態を示す aria-checked がありません。

```vue annotate="remove:2"
<template>
  <span role="checkbox">Receive updates</span>
</template>
```

<span id="a11y-role-has-required-aria-props-good"></span>

**良い**

`aria-checked="false"` を追加し、checkbox に必要な状態を指定します。

```vue annotate="add:2"
<template>
  <span role="checkbox" aria-checked="false">Receive updates</span>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/role_has_required_aria_props.rs#L30) · [全ルール](all.md)

### `a11y/tabindex-no-positive`

通常のフォーカス順序を変える正の tabindex を検出します。

[悪い例](#a11y-tabindex-no-positive-bad) · [良い例](#a11y-tabindex-no-positive-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/tabindex-no-positive": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-tabindex-no-positive-bad"></span>

**悪い**

正の tabindex 3 で、通常の操作要素より先に独自のフォーカス順を作っています。

```vue annotate="remove:2"
<template>
  <button tabindex="3">Save</button>
</template>
```

<span id="a11y-tabindex-no-positive-good"></span>

**良い**

正の tabindex を取り除き、button の標準のフォーカス順を使います。

```vue annotate="add:2"
<template>
  <button>Save</button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/tabindex_no_positive.rs#L16) · [全ルール](all.md)

### `a11y/use-list`

箇条書きに見えるテキストをリスト要素で表現します。

[悪い例](#a11y-use-list-bad) · [良い例](#a11y-use-list-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/use-list": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-use-list-bad"></span>

**悪い**

タスクを p と文字のハイフンで並べており、リストの要素を使っていません。

```vue annotate="remove:2,3"
<template>
  <p>- First task</p>
  <p>- Second task</p>
</template>
```

<span id="a11y-use-list-good"></span>

**良い**

同じタスクを ul と li に入れ、リストとして表します。

```vue annotate="add:2,3,4,5"
<template>
  <ul>
    <li>First task</li>
    <li>Second task</li>
  </ul>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/use_list.rs#L36) · [全ルール](all.md)

### `vue/use-unique-element-ids`

静的 ID の代わりに useId() で再利用可能な ID を生成します。

[悪い例](#vue-use-unique-element-ids-bad) · [良い例](#vue-use-unique-element-ids-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/use-unique-element-ids": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-use-unique-element-ids-bad"></span>

**悪い**

固定の `email` ID がコンポーネントの各インスタンスで重複し、複数表示時に label の参照先が曖昧になります。

```vue annotate="remove:2,3"
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

<span id="vue-use-unique-element-ids-good"></span>

**良い**

`useId()` の `emailId` を label の `for` と input の `id` の両方に binding します。

```vue annotate="add:1,2,3,4,5,6,8,9"
<script setup>
import { useId } from "vue";

const emailId = useId();
</script>

<template>
  <label :for="emailId">Email</label>
  <input :id="emailId" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_unique_element_ids.rs#L52) · [全ルール](all.md)
