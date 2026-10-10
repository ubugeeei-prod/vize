---
title: "HTML ルール"
---

# HTML ルール

このページに、各ルールの目的・前提・設定・完全な悪い例と良い例をまとめています。変更箇所は行の色で示し、コピーしたコードには完全なソースを保持します。各例に現在の対応範囲も明記しています。


| ルール | 例 | 目的 |
| --- | --- | --- |
| [`html/deprecated-attr`](#html-deprecated-attr) | [悪い](#html-deprecated-attr-bad) · [良い](#html-deprecated-attr-good) | 非推奨の HTML 属性を検出します。 |
| [`html/deprecated-element`](#html-deprecated-element) | [悪い](#html-deprecated-element-bad) · [良い](#html-deprecated-element-good) | 非推奨の HTML 要素を検出します。 |
| [`html/id-duplication`](#html-id-duplication) | [悪い](#html-id-duplication-bad) · [良い](#html-id-duplication-good) | 同じテンプレート内の ID 重複を検出します。 |
| [`html/no-consecutive-br`](#html-no-consecutive-br) | [悪い](#html-no-consecutive-br-bad) · [良い](#html-no-consecutive-br-good) | 連続する br 要素による余白指定を検出します。 |
| [`html/no-dupe-style-properties`](#html-no-dupe-style-properties) | [悪い](#html-no-dupe-style-properties-bad) · [良い](#html-no-dupe-style-properties-good) | 静的 style 属性内のプロパティ重複を検出します。 |
| [`html/no-duplicate-class`](#html-no-duplicate-class) | [悪い](#html-no-duplicate-class-bad) · [良い](#html-no-duplicate-class-good) | 静的 class 属性内のクラス名重複を検出します。 |
| [`html/no-duplicate-dt`](#html-no-duplicate-dt) | [悪い](#html-no-duplicate-dt-bad) · [良い](#html-no-duplicate-dt-good) | dl 内の dt の名前重複を検出します。 |
| [`html/no-empty-palpable-content`](#html-no-empty-palpable-content) | [悪い](#html-no-empty-palpable-content-bad) · [良い](#html-no-empty-palpable-content-good) | 可視コンテンツを期待する要素が空の場合に検出します。 |
| [`html/require-datetime`](#html-require-datetime) | [悪い](#html-require-datetime-bad) · [良い](#html-require-datetime-good) | time 要素に機械可読の datetime を指定します。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)

### `html/deprecated-attr`

非推奨の HTML 属性を検出します。

[悪い例](#html-deprecated-attr-bad) · [良い例](#html-deprecated-attr-good)

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
        "html/deprecated-attr": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-deprecated-attr-bad"></span>

**悪い**

p に見た目を指定する旧来の align 属性を使っています。

```vue annotate="remove:1,2,3"
<template>
<p align="center">Notice</p>
</template>
```

<span id="html-deprecated-attr-good"></span>

**良い**

クラスと text-align: center で、配置を CSS に移します。

```vue annotate="add:1,2"
<template><p class="notice">Notice</p></template>
<style scoped>.notice { text-align: center; }</style>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/deprecated_attr.rs#L32) · [全ルール](all.md)

### `html/deprecated-element`

非推奨の HTML 要素を検出します。

[悪い例](#html-deprecated-element-bad) · [良い例](#html-deprecated-element-good)

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
        "html/deprecated-element": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-deprecated-element-bad"></span>

**悪い**

旧来の見た目のための center 要素を使っています。

```vue annotate="remove:2"
<template>
  <center>Profile</center>
</template>
```

<span id="html-deprecated-element-good"></span>

**良い**

内容は残し、section とスタイル用のクラスに置き換えます。

```vue annotate="add:2"
<template>
  <section class="profile">Profile</section>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/deprecated_element.rs#L33) · [全ルール](all.md)

### `html/id-duplication`

同じテンプレート内の ID 重複を検出します。

[悪い例](#html-id-duplication-bad) · [良い例](#html-id-duplication-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "html/id-duplication": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-id-duplication-bad"></span>

**悪い**

入力欄と説明の p が同じ email の ID を使い、label の参照先が重複しています。

```vue annotate="remove:3,4"
<template>
  <label for="email">Email</label>
  <input id="email" />
  <p id="email">Required</p>
</template>
```

<span id="html-id-duplication-good"></span>

**良い**

入力欄は email、説明は email-help に分け、aria-describedby で説明を参照します。

```vue annotate="add:3,4"
<template>
  <label for="email">Email</label>
  <input id="email" aria-describedby="email-help" />
  <p id="email-help">Required</p>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/id_duplication.rs#L36) · [全ルール](all.md)

### `html/no-consecutive-br`

連続する br 要素による余白指定を検出します。

[悪い例](#html-no-consecutive-br-bad) · [良い例](#html-no-consecutive-br-good)

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
        "html/no-consecutive-br": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-consecutive-br-bad"></span>

**悪い**

一つの p の中に br を二つ続けて入れ、ブロック間の余白を作っています。

```vue annotate="remove:2"
<template>
  <p>First line<br /><br />Second block</p>
</template>
```

<span id="html-no-consecutive-br-good"></span>

**良い**

内容を別の p に分け、連続した br を使いません。

```vue annotate="add:2,3"
<template>
  <p>First line</p>
  <p>Second block</p>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_consecutive_br.rs#L30) · [全ルール](all.md)

### `html/no-dupe-style-properties`

静的 style 属性内のプロパティ重複を検出します。

[悪い例](#html-no-dupe-style-properties-bad) · [良い例](#html-no-dupe-style-properties-good)

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
        "html/no-dupe-style-properties": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-dupe-style-properties-bad"></span>

**悪い**

静的な style の中で同じプロパティを重複させています。margin と MARGIN も同じプロパティとして扱われます。

```vue annotate="remove:2,3"
<template>
<div style="color: red; color: blue">text</div>
<div style="margin: 0; MARGIN: 1px">text</div>
</template>
```

<span id="html-no-dupe-style-properties-good"></span>

**良い**

静的な style では color と background を分けます。動的な style バインディングはこの静的属性の検査の対象外です。

```vue annotate="add:2,3"
<template>
<div style="color: red; background: blue">text</div>
<div :style="{ color: a, color: b }">text</div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/html/no_dupe_style_properties.rs#L37) · [全ルール](all.md)

### `html/no-duplicate-class`

静的 class 属性内のクラス名重複を検出します。

[悪い例](#html-no-duplicate-class-bad) · [良い例](#html-no-duplicate-class-good)

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
        "html/no-duplicate-class": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-duplicate-class-bad"></span>

**悪い**

静的な class の中に btn が二回あります。

```vue annotate="remove:2"
<template>
<div class="btn btn primary">click</div>
</template>
```

<span id="html-no-duplicate-class-good"></span>

**良い**

btn は一回だけ残し、別の primary と合わせて指定します。

```vue annotate="add:2"
<template>
<div class="btn primary">click</div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/html/no_duplicate_class.rs#L32) · [全ルール](all.md)

### `html/no-duplicate-dt`

dl 内の dt の名前重複を検出します。

[悪い例](#html-no-duplicate-dt-bad) · [良い例](#html-no-duplicate-dt-good)

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
        "html/no-duplicate-dt": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-duplicate-dt-bad"></span>

**悪い**

同じ dl に API の用語を二回指定しています。

```vue annotate="remove:5"
<template>
  <dl>
    <dt>API</dt>
    <dd>Public interface</dd>
    <dt>API</dt>
    <dd>Internal service</dd>
  </dl>
</template>
```

<span id="html-no-duplicate-dt-good"></span>

**良い**

API の dt を一つにし、その後に二つの dd を並べます。

```vue
<template>
  <dl>
    <dt>API</dt>
    <dd>Public interface</dd>
    <dd>Internal service</dd>
  </dl>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_duplicate_dt.rs#L41) · [全ルール](all.md)

### `html/no-empty-palpable-content`

可視コンテンツを期待する要素が空の場合に検出します。

[悪い例](#html-no-empty-palpable-content-bad) · [良い例](#html-no-empty-palpable-content-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: [型付きオプションと既定値](options.md)を参照してください。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "html/no-empty-palpable-content": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-empty-palpable-content-bad"></span>

**悪い**

p、li、td がいずれも空で、意味のある内容がありません。

```vue annotate="remove:2,3,4"
<template>
  <p></p>
  <li></li>
  <td></td>
</template>
```

<span id="html-no-empty-palpable-content-good"></span>

**良い**

p は文字、li は補間で内容を入れ、空の td には aria-label で名前を指定します。

```vue annotate="add:2,3,4"
<template>
  <p>Overview</p>
  <li>{{ item.label }}</li>
  <td aria-label="No value"></td>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_empty_palpable_content.rs#L32) · [全ルール](all.md)

### `html/require-datetime`

time 要素に機械可読の datetime を指定します。

[悪い例](#html-require-datetime-bad) · [良い例](#html-require-datetime-good)

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
        "html/require-datetime": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-require-datetime-bad"></span>

**悪い**

time に人が読む日付だけがあり、機械が読む datetime がありません。

```vue annotate="remove:2"
<template>
  <time>May 13, 2026</time>
</template>
```

<span id="html-require-datetime-good"></span>

**良い**

`datetime="2026-05-13"` に対応する日付を指定します。

```vue annotate="add:2"
<template>
  <time datetime="2026-05-13">May 13, 2026</time>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/require_datetime.rs#L34) · [全ルール](all.md)
