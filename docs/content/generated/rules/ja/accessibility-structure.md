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
| [`a11y/heading-has-content`](#a11y-heading-has-content) | [悪い](#a11y-heading-has-content-bad) · [良い](#a11y-heading-has-content-good) | 見出しに支援技術で読める内容を用意します。 |
| [`a11y/heading-levels`](#a11y-heading-levels) | [悪い](#a11y-heading-levels-bad) · [良い](#a11y-heading-levels-good) | 見出しの階層を飛ばした指定を検出します。 |
| [`a11y/iframe-has-title`](#a11y-iframe-has-title) | [悪い](#a11y-iframe-has-title-bad) · [良い](#a11y-iframe-has-title-good) | iframe に内容を説明する title を指定します。 |
| [`a11y/img-alt`](#a11y-img-alt) | [悪い](#a11y-img-alt-bad) · [良い](#a11y-img-alt-good) | 画像に alt 属性を指定します。装飾画像は空の alt を使います。 |
| [`a11y/interactive-supports-focus`](#a11y-interactive-supports-focus) | [悪い](#a11y-interactive-supports-focus-bad) · [良い](#a11y-interactive-supports-focus-good) | 操作可能な role の要素をフォーカス可能にします。 |
| [`a11y/label-has-for`](#a11y-label-has-for) | [悪い](#a11y-label-has-for-bad) · [良い](#a11y-label-has-for-good) | label を対象のフォーム部品と関連付けます。 |
| [`a11y/landmark-roles`](#a11y-landmark-roles) | [悪い](#a11y-landmark-roles-bad) · [良い](#a11y-landmark-roles-good) | ランドマーク role の配置と重複を検査します。 |
| [`a11y/media-has-caption`](#a11y-media-has-caption) | [悪い](#a11y-media-has-caption-bad) · [良い](#a11y-media-has-caption-good) | 音声・動画に字幕を用意します。 |

[全ルール](./all.md) · [ルールのオプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)

### `a11y/heading-has-content`

見出しに支援技術で読める内容を用意します。

[悪い例](#a11y-heading-has-content-bad) · [良い例](#a11y-heading-has-content-good)

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

`h2` でレベル 2 の見出しを宣言していますが、見出しの内容が空です。

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

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/heading_has_content.rs#L17) · [全ルール](all.md)

### `a11y/heading-levels`

見出しの階層を飛ばした指定を検出します。

[悪い例](#a11y-heading-levels-bad) · [良い例](#a11y-heading-levels-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
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

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/heading_levels.rs#L34) · [全ルール](all.md)

### `a11y/iframe-has-title`

iframe に内容を説明する title を指定します。

[悪い例](#a11y-iframe-has-title-bad) · [良い例](#a11y-iframe-has-title-good)

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

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/iframe_has_title.rs#L15) · [全ルール](all.md)

### `a11y/img-alt`

画像に alt 属性を指定します。装飾画像は空の alt を使います。

[悪い例](#a11y-img-alt-bad) · [良い例](#a11y-img-alt-good)

既定の重大度: `warning`  
プリセット: _none_  
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

アバター画像に `alt` 属性がなく、代替テキストが指定されていません。

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

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/img_alt.rs#L16) · [全ルール](all.md)

### `a11y/interactive-supports-focus`

操作可能な role の要素をフォーカス可能にします。

[悪い例](#a11y-interactive-supports-focus-bad) · [良い例](#a11y-interactive-supports-focus-good)

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

span に button のロールとクリックハンドラーを付けても、キーボードでフォーカスできる要素にはなりません。

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

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/interactive_supports_focus.rs#L31) · [全ルール](all.md)

### `a11y/label-has-for`

label を対象のフォーム部品と関連付けます。

[悪い例](#a11y-label-has-for-bad) · [良い例](#a11y-label-has-for-good)

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

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/label_has_for.rs#L27) · [全ルール](all.md)

### `a11y/landmark-roles`

ランドマーク role の配置と重複を検査します。

[悪い例](#a11y-landmark-roles-bad) · [良い例](#a11y-landmark-roles-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
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

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/landmark_roles.rs#L44) · [全ルール](all.md)

### `a11y/media-has-caption`

音声・動画に字幕を用意します。

[悪い例](#a11y-media-has-caption-bad) · [良い例](#a11y-media-has-caption-good)

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

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/media_has_caption.rs#L30) · [全ルール](all.md)
