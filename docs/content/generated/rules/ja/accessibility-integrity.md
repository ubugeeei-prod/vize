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
| [`a11y/no-role-presentation-on-focusable`](#a11y-no-role-presentation-on-focusable) | [悪い](#a11y-no-role-presentation-on-focusable-bad) · [良い](#a11y-no-role-presentation-on-focusable-good) | フォーカス可能な要素への role="presentation" または role="none" の指定を検出します。 |
| [`a11y/no-static-element-interactions`](#a11y-no-static-element-interactions) | [悪い](#a11y-no-static-element-interactions-bad) · [良い](#a11y-no-static-element-interactions-good) | 操作部品ではない要素へのイベント指定を検出します。 |
| [`a11y/placeholder-label-option`](#a11y-placeholder-label-option) | [悪い](#a11y-placeholder-label-option-bad) · [良い](#a11y-placeholder-label-option-good) | select のプレースホルダー option に disabled または hidden を指定します。 |
| [`a11y/role-has-required-aria-props`](#a11y-role-has-required-aria-props) | [悪い](#a11y-role-has-required-aria-props-bad) · [良い](#a11y-role-has-required-aria-props-good) | ARIA role が必要とする属性を指定します。 |
| [`a11y/tabindex-no-positive`](#a11y-tabindex-no-positive) | [悪い](#a11y-tabindex-no-positive-bad) · [良い](#a11y-tabindex-no-positive-good) | 通常のフォーカス順序を変える正の tabindex を検出します。 |
| [`a11y/use-list`](#a11y-use-list) | [悪い](#a11y-use-list-bad) · [良い](#a11y-use-list-good) | 箇条書きに見えるテキストをリスト要素で表現します。 |
| [`vue/use-unique-element-ids`](#vue-use-unique-element-ids) | [悪い](#vue-use-unique-element-ids-bad) · [良い](#vue-use-unique-element-ids-good) | 静的 ID の代わりに useId() で一意な ID を生成します。 |

[全ルール](./all.md) · [ルールのオプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)

### `a11y/no-role-presentation-on-focusable`

フォーカス可能な要素への role="presentation" または role="none" の指定を検出します。

[悪い例](#a11y-no-role-presentation-on-focusable-bad) · [良い例](#a11y-no-role-presentation-on-focusable-good)

既定の重大度: `error`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
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

フォーカスできる Billing のリンクに role="presentation" を指定しています。リンク本来の操作可能な役割と矛盾するため、ブラウザーはこの presentation の指定を無視する必要があります。

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

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_role_presentation_on_focusable.rs#L19) · [全ルール](all.md)

### `a11y/no-static-element-interactions`

操作部品ではない要素へのイベント指定を検出します。

[悪い例](#a11y-no-static-element-interactions-bad) · [良い例](#a11y-no-static-element-interactions-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
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

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_static_element_interactions.rs#L31) · [全ルール](all.md)

### `a11y/placeholder-label-option`

select のプレースホルダー option に disabled または hidden を指定します。

[悪い例](#a11y-placeholder-label-option-bad) · [良い例](#a11y-placeholder-label-option-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
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

disabled を追加して案内の option を選択できなくし、選択可能な Japan の option と区別します。

```vue annotate="add:3"
<template>
  <select v-model="country">
    <option value="" disabled>Choose a country</option>
    <option value="jp">Japan</option>
  </select>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/placeholder_label_option.rs#L36) · [全ルール](all.md)

### `a11y/role-has-required-aria-props`

ARIA role が必要とする属性を指定します。

[悪い例](#a11y-role-has-required-aria-props-bad) · [良い例](#a11y-role-has-required-aria-props-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
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

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/role_has_required_aria_props.rs#L30) · [全ルール](all.md)

### `a11y/tabindex-no-positive`

通常のフォーカス順序を変える正の tabindex を検出します。

[悪い例](#a11y-tabindex-no-positive-bad) · [良い例](#a11y-tabindex-no-positive-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
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

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/tabindex_no_positive.rs#L16) · [全ルール](all.md)

### `a11y/use-list`

箇条書きに見えるテキストをリスト要素で表現します。

[悪い例](#a11y-use-list-bad) · [良い例](#a11y-use-list-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
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

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/use_list.rs#L36) · [全ルール](all.md)

### `vue/use-unique-element-ids`

静的 ID の代わりに useId() で一意な ID を生成します。

[悪い例](#vue-use-unique-element-ids-bad) · [良い例](#vue-use-unique-element-ids-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
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

`useId()` の `emailId` を label の `for` と input の `id` の両方にバインドします。

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

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_unique_element_ids.rs#L52) · [全ルール](all.md)
