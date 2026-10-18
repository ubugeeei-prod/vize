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
| [`a11y/alt-text`](#a11y-alt-text) | [悪い](#a11y-alt-text-bad) · [良い](#a11y-alt-text-good) | 画像などのメディアに代替テキストを用意します。 |
| [`a11y/anchor-has-content`](#a11y-anchor-has-content) | [悪い](#a11y-anchor-has-content-bad) · [良い](#a11y-anchor-has-content-good) | リンクに支援技術で読める内容を用意します。 |
| [`a11y/anchor-is-valid`](#a11y-anchor-is-valid) | [悪い](#a11y-anchor-is-valid-bad) · [良い](#a11y-anchor-is-valid-good) | リンクの href に有効な移動先を指定します。 |
| [`a11y/aria-props`](#a11y-aria-props) | [悪い](#a11y-aria-props-bad) · [良い](#a11y-aria-props-good) | 存在しない ARIA 属性を検出します。 |
| [`a11y/aria-role`](#a11y-aria-role) | [悪い](#a11y-aria-role-bad) · [良い](#a11y-aria-role-good) | 抽象ロールを除く、有効な ARIA role を指定します。 |
| [`a11y/aria-unsupported-elements`](#a11y-aria-unsupported-elements) | [悪い](#a11y-aria-unsupported-elements-bad) · [良い](#a11y-aria-unsupported-elements-good) | ARIA 属性を使用できない要素への指定を検出します。 |
| [`a11y/click-events-have-key-events`](#a11y-click-events-have-key-events) | [悪い](#a11y-click-events-have-key-events-bad) · [良い](#a11y-click-events-have-key-events-good) | クリックで操作する要素にキーボード操作も用意します。 |
| [`a11y/form-control-has-label`](#a11y-form-control-has-label) | [悪い](#a11y-form-control-has-label-bad) · [良い](#a11y-form-control-has-label-good) | フォーム部品に関連付けられたラベルを用意します。 |

[全ルール](./all.md) · [ルールのオプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)

### `a11y/alt-text`

画像などのメディアに代替テキストを用意します。

[悪い例](#a11y-alt-text-bad) · [良い例](#a11y-alt-text-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
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

`alt="Submit search"` を追加し、検索を送信する操作を説明する名前を支援技術に伝えます。

```vue annotate="add:2"
<template>
  <input type="image" src="/submit.png" alt="Submit search" />
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/alt_text.rs#L33) · [全ルール](all.md)

### `a11y/anchor-has-content`

リンクに支援技術で読める内容を用意します。

[悪い例](#a11y-anchor-has-content-bad) · [良い例](#a11y-anchor-has-content-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
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

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_has_content.rs#L16) · [全ルール](all.md)

### `a11y/anchor-is-valid`

リンクの href に有効な移動先を指定します。

[悪い例](#a11y-anchor-is-valid-bad) · [良い例](#a11y-anchor-is-valid-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
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

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_is_valid.rs#L30) · [全ルール](all.md)

### `a11y/aria-props`

存在しない ARIA 属性を検出します。

[悪い例](#a11y-aria-props-bad) · [良い例](#a11y-aria-props-good)

既定の重大度: `error`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
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

`aria-lable` は綴りが誤っており、有効な ARIA 属性ではありません。

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

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_props.rs#L18) · [全ルール](all.md)

### `a11y/aria-role`

抽象ロールを除く、有効な ARIA role を指定します。

[悪い例](#a11y-aria-role-bad) · [良い例](#a11y-aria-role-good)

既定の重大度: `error`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
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

`datepicker` は有効な ARIA ロールではありません。

```vue annotate="remove:2"
<template>
  <section role="datepicker">...</section>
</template>
```

<span id="a11y-aria-role-good"></span>

**良い**

有効な ARIA ロールである `dialog` を使い、日付を選択する領域の名前も指定します。

```vue annotate="add:2"
<template>
  <section role="dialog" aria-label="Choose a date">...</section>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_role.rs#L21) · [全ルール](all.md)

### `a11y/aria-unsupported-elements`

ARIA 属性を使用できない要素への指定を検出します。

[悪い例](#a11y-aria-unsupported-elements-bad) · [良い例](#a11y-aria-unsupported-elements-good)

既定の重大度: `error`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
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

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_unsupported_elements.rs#L18) · [全ルール](all.md)

### `a11y/click-events-have-key-events`

クリックで操作する要素にキーボード操作も用意します。

[悪い例](#a11y-click-events-have-key-events-bad) · [良い例](#a11y-click-events-have-key-events-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
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

通常の `div` にクリックハンドラーだけを指定し、キーボード操作に対応していません。

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

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/click_events_have_key_events.rs#L17) · [全ルール](all.md)

### `a11y/form-control-has-label`

フォーム部品に関連付けられたラベルを用意します。

[悪い例](#a11y-form-control-has-label-bad) · [良い例](#a11y-form-control-has-label-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
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

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/form_control_has_label.rs#L19) · [全ルール](all.md)
