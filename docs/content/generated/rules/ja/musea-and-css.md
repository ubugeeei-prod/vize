---
title: "Musea と CSS のルール"
---

# Musea と CSS のルール

このページに、各ルールの目的・前提・設定・完全な悪い例と良い例をまとめています。変更箇所は行の色で示し、コピーしたコードには完全なソースを保持します。各例に現在の対応範囲も明記しています。


| ルール | 例 | 目的 |
| --- | --- | --- |
| [`css/no-display-none`](#css-no-display-none) | [悪い](#css-no-display-none-bad) · [良い](#css-no-display-none-good) | 表示切り替えに display: none を使う箇所で v-show を検討します。 |
| [`css/no-hardcoded-values`](#css-no-hardcoded-values) | [悪い](#css-no-hardcoded-values-bad) · [良い](#css-no-hardcoded-values-good) | CSS の直接指定値を CSS 変数にまとめます。 |
| [`css/no-id-selectors`](#css-no-id-selectors) | [悪い](#css-no-id-selectors-bad) · [良い](#css-no-id-selectors-good) | 詳細度が高い CSS の ID セレクターを検出します。 |
| [`css/no-important`](#css-no-important) | [悪い](#css-no-important-bad) · [良い](#css-no-important-good) | 通常のカスケードを上書きする !important を検出します。 |
| [`css/no-utility-classes`](#css-no-utility-classes) | [悪い](#css-no-utility-classes-bad) · [良い](#css-no-utility-classes-good) | コンポーネント内で utility class を定義する箇所を検出します。 |
| [`css/no-v-bind-performance`](#css-no-v-bind-performance) | [悪い](#css-no-v-bind-performance-bad) · [良い](#css-no-v-bind-performance-good) | CSS v-bind() の実行時コストを検討するための警告です。 |
| [`css/prefer-logical-properties`](#css-prefer-logical-properties) | [悪い](#css-prefer-logical-properties-bad) · [良い](#css-prefer-logical-properties-good) | 書字方向に対応する CSS の論理プロパティを使います。 |
| [`css/prefer-nested-selectors`](#css-prefer-nested-selectors) | [悪い](#css-prefer-nested-selectors-bad) · [良い](#css-prefer-nested-selectors-good) | 子孫セレクターを CSS nesting でまとめます。 |
| [`css/prefer-slotted`](#css-prefer-slotted) | [悪い](#css-prefer-slotted-bad) · [良い](#css-prefer-slotted-good) | slot や子コンポーネントに対する scoped CSS のセレクターを検査します。 |
| [`css/require-font-display`](#css-require-font-display) | [悪い](#css-require-font-display-bad) · [良い](#css-require-font-display-good) | @font-face に font-display を指定します。 |
| [`musea/no-empty-variant`](#musea-no-empty-variant) | [悪い](#musea-no-empty-variant-bad) · [良い](#musea-no-empty-variant-good) | 内容のない variant ブロックを検出します。 |
| [`musea/prefer-design-tokens`](#musea-prefer-design-tokens) | [悪い](#musea-prefer-design-tokens-bad) · [良い](#musea-prefer-design-tokens-good) | 登録した design token に一致する直接指定値を CSS 変数で表現します。 |
| [`musea/require-component`](#musea-require-component) | [悪い](#musea-require-component-bad) · [良い](#musea-require-component-good) | art ブロックに対象の component を指定します。 |
| [`musea/require-title`](#musea-require-title) | [悪い](#musea-require-title-bad) · [良い](#musea-require-title-good) | art ブロックに title を指定します。 |
| [`musea/unique-variant-names`](#musea-unique-variant-names) | [悪い](#musea-unique-variant-names-bad) · [良い](#musea-unique-variant-names-good) | 同じ Art ファイル内の variant 名を一意にします。 |
| [`musea/valid-variant`](#musea-valid-variant) | [悪い](#musea-valid-variant-bad) · [良い](#musea-valid-variant-good) | variant ブロックに name を指定します。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)

### `css/no-display-none`

表示切り替えに display: none を使う箇所で v-show を検討します。

[悪い例](#css-no-display-none-bad) · [良い例](#css-no-display-none-good)

既定の重大度: `warning`  
プリセット: `opinionated`, `nuxt`  
自動修正: なし。修正内容を確認してください  
適用範囲: SFC の style ブロック内の CSS  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-display-none": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-display-none-bad"></span>

**悪い**

ローカルの p を `.message` の CSS で非表示にし、テンプレートに表示条件を指定していません。

```vue annotate="remove:2,4,5,6,7,8,9"
<template>
  <p class="message">Saved</p>
</template>

<style scoped>
.message {
  display: none;
}
</style>
```

<span id="css-no-display-none-good"></span>

**良い**

同じ p に `v-show="isSaved"` で表示条件を指定し、display: none を取り除きます。

```vue annotate="add:2"
<template>
  <p v-show="isSaved" class="message">Saved</p>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_display_none.rs#L27) · [全ルール](all.md)

### `css/no-hardcoded-values`

CSS の直接指定値を CSS 変数にまとめます。

[悪い例](#css-no-hardcoded-values-bad) · [良い例](#css-no-hardcoded-values-good)

既定の重大度: `warning`  
プリセット: `opinionated`, `nuxt`  
自動修正: なし。修正内容を確認してください  
適用範囲: SFC の style ブロック内の CSS  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-hardcoded-values": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-hardcoded-values-bad"></span>

**悪い**

button の余白と色に数値や 16 進の色を直接指定しています。

```vue annotate="remove:3,4"
<style scoped>
.button {
  padding: 12px 16px;
  color: #174ea6;
}
</style>
```

<span id="css-no-hardcoded-values-good"></span>

**良い**

余白と色を名前付きのカスタムプロパティで参照し、トークンとして管理できる形にします。

```vue annotate="add:3,4"
<style scoped>
.button {
  padding: var(--space-3) var(--space-4);
  color: var(--color-action-text);
}
</style>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_hardcoded_values.rs#L30) · [全ルール](all.md)

### `css/no-id-selectors`

詳細度が高い CSS の ID セレクターを検出します。

[悪い例](#css-no-id-selectors-bad) · [良い例](#css-no-id-selectors-good)

既定の重大度: `warning`  
プリセット: `opinionated`, `nuxt`  
自動修正: なし。修正内容を確認してください  
適用範囲: SFC の style ブロック内の CSS  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-id-selectors": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-id-selectors-bad"></span>

**悪い**

`#submit` で ID セレクターにスタイルを結び付けています。

```vue annotate="remove:2"
<style scoped>
#submit {
  font-weight: 600;
}
</style>
```

<span id="css-no-id-selectors-good"></span>

**良い**

再利用できる `.submit` のクラスを使い、ID セレクターを取り除きます。

```vue annotate="add:2"
<style scoped>
.submit {
  font-weight: 600;
}
</style>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_id_selectors.rs#L19) · [全ルール](all.md)

### `css/no-important`

通常のカスケードを上書きする !important を検出します。

[悪い例](#css-no-important-bad) · [良い例](#css-no-important-good)

既定の重大度: `warning`  
プリセット: `opinionated`, `nuxt`  
自動修正: なし。修正内容を確認してください  
適用範囲: SFC の style ブロック内の CSS  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-important": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-important-bad"></span>

**悪い**

color に `!important` を付け、通常のカスケードの優先順位を上書きしています。

```vue annotate="remove:3"
<style scoped>
.button {
  color: red !important;
}
</style>
```

<span id="css-no-important-good"></span>

**良い**

important を使わず、カスタムプロパティから色を参照します。

```vue annotate="add:3"
<style scoped>
.button {
  color: var(--button-color);
}
</style>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_important.rs#L14) · [全ルール](all.md)

### `css/no-utility-classes`

コンポーネント内で utility class を定義する箇所を検出します。

[悪い例](#css-no-utility-classes-bad) · [良い例](#css-no-utility-classes-good)

既定の重大度: `warning`  
プリセット: `opinionated`, `nuxt`  
自動修正: なし。修正内容を確認してください  
適用範囲: SFC の style ブロック内の CSS  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-utility-classes": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-utility-classes-bad"></span>

**悪い**

`.flex`、`.mt-4`、`.text-center` のように、個々の見た目を名前にしたクラスを定義しています。

```vue annotate="remove:2,3,4"
<style scoped>
.flex { display: flex; }
.mt-4 { margin-top: 1rem; }
.text-center { text-align: center; }
</style>
```

<span id="css-no-utility-classes-good"></span>

**良い**

コンポーネント固有の `.my-component` にスタイルをまとめます。

```vue annotate="add:2"
<style scoped>
.my-component { display: flex; margin-top: 1rem; }
</style>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_utility_classes.rs#L37) · [全ルール](all.md)

### `css/no-v-bind-performance`

CSS v-bind() の実行時コストを検討するための警告です。

[悪い例](#css-no-v-bind-performance-bad) · [良い例](#css-no-v-bind-performance-good)

既定の重大度: `warning`  
プリセット: `opinionated`, `nuxt`  
自動修正: なし。修正内容を確認してください  
適用範囲: SFC の style ブロック内の CSS  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-v-bind-performance": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-v-bind-performance-bad"></span>

**悪い**

変化する offset を SFC の CSS の v-bind() で参照しています。

```vue annotate="remove:1,2,3,4,5"
<style scoped>
.card {
  transform: translateX(v-bind(offset));
}
</style>
```

<span id="css-no-v-bind-performance-good"></span>

**良い**

変化する transform を要素の style バインディングに直接指定します。

```vue annotate="add:1,2,3"
<template>
  <article :style="{ transform: `translateX(${offset}px)` }" class="card" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_v_bind_performance.rs#L20) · [全ルール](all.md)

### `css/prefer-logical-properties`

書字方向に対応する CSS の論理プロパティを使います。

[悪い例](#css-prefer-logical-properties-bad) · [良い例](#css-prefer-logical-properties-good)

既定の重大度: `warning`  
プリセット: `opinionated`, `nuxt`  
自動修正: なし。修正内容を確認してください  
適用範囲: SFC の style ブロック内の CSS  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/prefer-logical-properties": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-prefer-logical-properties-bad"></span>

**悪い**

margin-left は文字の方向に関係なく物理的な左側を指定します。

```vue annotate="remove:3"
<style scoped>
.panel {
  margin-left: 1rem;
}
</style>
```

<span id="css-prefer-logical-properties-good"></span>

**良い**

margin-inline-start を使い、インライン方向の開始側に余白を指定します。

```vue annotate="add:3"
<style scoped>
.panel {
  margin-inline-start: 1rem;
}
</style>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_logical_properties.rs#L15) · [全ルール](all.md)

### `css/prefer-nested-selectors`

子孫セレクターを CSS nesting でまとめます。

[悪い例](#css-prefer-nested-selectors-bad) · [良い例](#css-prefer-nested-selectors-good)

既定の重大度: `warning`  
プリセット: `opinionated`, `nuxt`  
自動修正: なし。修正内容を確認してください  
適用範囲: SFC の style ブロック内の CSS  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/prefer-nested-selectors": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-prefer-nested-selectors-bad"></span>

**悪い**

平らなルールの `.card .title` に、親のセレクターを含めて指定しています。

```vue annotate="remove:2"
<style scoped>
.card .title { color: red; }
</style>
```

<span id="css-prefer-nested-selectors-good"></span>

**良い**

`.card` の中に `.title` を入れ、親子のスタイルを一緒に管理します。

```vue annotate="add:2"
<style scoped>
.card { .title { color: red; } }
</style>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_nested_selectors.rs#L14) · [全ルール](all.md)

### `css/prefer-slotted`

slot や子コンポーネントに対する scoped CSS のセレクターを検査します。

[悪い例](#css-prefer-slotted-bad) · [良い例](#css-prefer-slotted-good)

既定の重大度: `warning`  
プリセット: `opinionated`, `nuxt`  
自動修正: なし。修正内容を確認してください  
適用範囲: SFC の style ブロック内の CSS  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/prefer-slotted": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-prefer-slotted-bad"></span>

**悪い**

scoped の CSS で、渡された要素ではなく slot の出口を対象にしています。

```vue annotate="remove:2"
<style scoped>
slot { color: red; }
</style>
```

<span id="css-prefer-slotted-good"></span>

**良い**

:slotted(.label) を使い、slot から渡される label の要素を対象にします。

```vue annotate="add:2"
<style scoped>
:slotted(.label) { color: red; }
</style>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_slotted.rs#L34) · [全ルール](all.md)

### `css/require-font-display`

@font-face に font-display を指定します。

[悪い例](#css-require-font-display-bad) · [良い例](#css-require-font-display-good)

既定の重大度: `warning`  
プリセット: `opinionated`, `nuxt`  
自動修正: なし。修正内容を確認してください  
適用範囲: SFC の style ブロック内の CSS  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/require-font-display": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-require-font-display-bad"></span>

**悪い**

font-face にフォントの参照先はありますが、font-display の方針を指定していません。

```vue
<style>
@font-face {
  font-family: "Inter";
  src: url("/inter.woff2") format("woff2");
}
</style>
```

<span id="css-require-font-display-good"></span>

**良い**

font-display: swap を追加し、フォールバックからフォントを表示する方針を指定します。

```vue annotate="add:5"
<style>
@font-face {
  font-family: "Inter";
  src: url("/inter.woff2") format("woff2");
  font-display: swap;
}
</style>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/require_font_display.rs#L13) · [全ルール](all.md)

### `musea/no-empty-variant`

内容のない variant ブロックを検出します。

[悪い例](#musea-no-empty-variant-bad) · [良い例](#musea-no-empty-variant-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Musea の .art.vue ファイルの art / variant / style  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/no-empty-variant": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-no-empty-variant-bad"></span>

**悪い**

primary の名前がある variant が空で、プレビューする内容がありません。

```vue annotate="remove:2"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

<span id="musea-no-empty-variant-good"></span>

**良い**

variant の中に primary の Button と Save の内容を入れます。

```vue annotate="add:2,3,4"
<art title="Button" component="./Button.vue">
  <variant name="primary">
    <Button tone="primary">Save</Button>
  </variant>
</art>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/no_empty_variant.rs#L8) · [全ルール](all.md)

### `musea/prefer-design-tokens`

登録した design token に一致する直接指定値を CSS 変数で表現します。

[悪い例](#musea-prefer-design-tokens-bad) · [良い例](#musea-prefer-design-tokens-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Musea の .art.vue ファイルの art / variant / style  
オプション: [型付きオプションと既定値](options.md)を参照してください。

.art.vue ファイルと下記の token 一覧が必要です。任意の色から token を推測するルールではありません。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/prefer-design-tokens": "warn"
      },
      "ruleOptions": {
        "musea/prefer-design-tokens": {
          "tokens": [
            {
              "path": "color.primary",
              "value": "#3b82f6",
              "tier": "semantic"
            }
          ]
        }
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-prefer-design-tokens-bad"></span>

**悪い**

art の例で、設定した primary のデザイントークンではなく、青の色を直接指定しています。

`Button.art.vue`

```vue annotate="remove:6"
<art title="Button" component="Button">
<variant name="Primary"><Button /></variant>
</art>
<style scoped>
.button {
  color: #3b82f6;
}
</style>
```

<span id="musea-prefer-design-tokens-good"></span>

**良い**

この例で設定する --color-primary のトークンを参照します。

`Button.art.vue`

```vue annotate="add:6"
<art title="Button" component="Button">
<variant name="Primary"><Button /></variant>
</art>
<style scoped>
.button {
  color: var(--color-primary);
}
</style>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/prefer_design_tokens.rs#L32) · [全ルール](all.md)

### `musea/require-component`

art ブロックに対象の component を指定します。

[悪い例](#musea-require-component-bad) · [良い例](#musea-require-component-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Musea の .art.vue ファイルの art / variant / style  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/require-component": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-require-component-bad"></span>

**悪い**

art に title はありますが、プレビューする component の指定がありません。

```vue annotate="remove:1"
<art title="Button">
  <variant name="primary" />
</art>
```

<span id="musea-require-component-good"></span>

**良い**

defineArt で ./Button.vue を art の component として指定します。

```vue annotate="add:1,2,3,4,5"
<script setup>
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="primary" />
</art>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_component.rs#L11) · [全ルール](all.md)

### `musea/require-title`

art ブロックに title を指定します。

[悪い例](#musea-require-title-bad) · [良い例](#musea-require-title-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Musea の .art.vue ファイルの art / variant / style  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/require-title": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-require-title-bad"></span>

**悪い**

art に Button.vue の指定はありますが、title がありません。

```vue annotate="remove:1"
<art component="./Button.vue">
  <variant name="primary" />
</art>
```

<span id="musea-require-title-good"></span>

**良い**

defineArt のオプションに Button の title を指定します。

```vue annotate="add:1,2,3,4,5"
<script setup>
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="primary" />
</art>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_title.rs#L31) · [全ルール](all.md)

### `musea/unique-variant-names`

同じ Art ファイル内の variant 名を一意にします。

[悪い例](#musea-unique-variant-names-bad) · [良い例](#musea-unique-variant-names-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Musea の .art.vue ファイルの art / variant / style  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/unique-variant-names": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-unique-variant-names-bad"></span>

**悪い**

同じ art の二つの variant に primary の名前を使っています。

```vue annotate="remove:3"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
  <variant name="primary" />
</art>
```

<span id="musea-unique-variant-names-good"></span>

**良い**

primary と secondary の別々の名前を指定します。

```vue annotate="add:3"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
  <variant name="secondary" />
</art>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/unique_variant_names.rs#L10) · [全ルール](all.md)

### `musea/valid-variant`

variant ブロックに name を指定します。

[悪い例](#musea-valid-variant-bad) · [良い例](#musea-valid-variant-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Musea の .art.vue ファイルの art / variant / style  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/valid-variant": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-valid-variant-bad"></span>

**悪い**

プレビューを識別する variant の name がありません。

```vue annotate="remove:2"
<art title="Button" component="./Button.vue">
  <variant />
</art>
```

<span id="musea-valid-variant-good"></span>

**良い**

variant に primary の name を指定します。

```vue annotate="add:2"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/valid_variant.rs#L8) · [全ルール](all.md)
