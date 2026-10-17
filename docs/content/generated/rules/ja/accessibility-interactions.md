---
title: "操作とフォーカスのアクセシビリティ"
---

<span id="accessibility-interactions"></span>

# 操作とフォーカスのアクセシビリティ

このルール群では、キーボード操作に関わる指定、フォーカス、マウスのイベント処理、支援技術から隠す要素、要素同士の参照を確認します。
独自の操作部品を追加したり、表示の切り替えやイベントの処理を変更したりするときに確認してください。

たとえば、[フォーカスと aria-hidden の関係](./reference/a11y-no-aria-hidden-on-focusable.md)では、フォーカスできる要素を支援技術から隠してしまう指定の例を確認できます。
悪い例と良い例を比較し、掲載された設定を使って、`vp run lint` を実行してください。
[アクセシビリティの全ルール](./accessibility.md)では、関連するルールも確認できます。

このページに、各ルールの目的・前提・設定と、省略のない悪い例・良い例をまとめています。変更箇所は行の色で示します。コードをコピーすると、ソースコード全体を取得できます。各例に現在の対応範囲も明記しています。


| ルール | 例 | 目的 |
| --- | --- | --- |
| [`a11y/mouse-events-have-key-events`](#a11y-mouse-events-have-key-events) | [悪い](#a11y-mouse-events-have-key-events-bad) · [良い](#a11y-mouse-events-have-key-events-good) | マウス操作と対応する focus / blur 操作を用意します。 |
| [`a11y/no-access-key`](#a11y-no-access-key) | [悪い](#a11y-no-access-key-bad) · [良い](#a11y-no-access-key-good) | 環境のショートカットと衝突し得る accesskey を検出します。 |
| [`a11y/no-aria-hidden-on-focusable`](#a11y-no-aria-hidden-on-focusable) | [悪い](#a11y-no-aria-hidden-on-focusable-bad) · [良い](#a11y-no-aria-hidden-on-focusable-good) | フォーカス可能な要素を aria-hidden で支援技術から隠す指定を検出します。 |
| [`a11y/no-autofocus`](#a11y-no-autofocus) | [悪い](#a11y-no-autofocus-bad) · [良い](#a11y-no-autofocus-good) | 意図せずフォーカスを移動させる autofocus を検出します。 |
| [`a11y/no-distracting-elements`](#a11y-no-distracting-elements) | [悪い](#a11y-no-distracting-elements-bad) · [良い](#a11y-no-distracting-elements-good) | marquee や blink などの注意をそらす要素を検出します。 |
| [`a11y/no-i-for-icon`](#a11y-no-i-for-icon) | [悪い](#a11y-no-i-for-icon-bad) · [良い](#a11y-no-i-for-icon-good) | アイコン用の i 要素を検出し、意味に合う要素を勧めます。 |
| [`a11y/no-redundant-roles`](#a11y-no-redundant-roles) | [悪い](#a11y-no-redundant-roles-bad) · [良い](#a11y-no-redundant-roles-good) | 要素本来の意味と重複する ARIA role を検出します。 |
| [`a11y/no-refer-to-non-existent-id`](#a11y-no-refer-to-non-existent-id) | [悪い](#a11y-no-refer-to-non-existent-id-bad) · [良い](#a11y-no-refer-to-non-existent-id-good) | 文書内に存在しない ID への参照を検出します。 |

[全ルール](./all.md) · [ルールのオプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)

### `a11y/mouse-events-have-key-events`

マウス操作と対応する focus / blur 操作を用意します。

[悪い例](#a11y-mouse-events-have-key-events-bad) · [良い例](#a11y-mouse-events-have-key-events-good)

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

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/mouse_events_have_key_events.rs#L30) · [全ルール](all.md)

### `a11y/no-access-key`

環境のショートカットと衝突し得る accesskey を検出します。

[悪い例](#a11y-no-access-key-bad) · [良い例](#a11y-no-access-key-good)

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

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_access_key.rs#L19) · [全ルール](all.md)

### `a11y/no-aria-hidden-on-focusable`

フォーカス可能な要素を aria-hidden で支援技術から隠す指定を検出します。

[悪い例](#a11y-no-aria-hidden-on-focusable-bad) · [良い例](#a11y-no-aria-hidden-on-focusable-good)

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

aria-hidden を取り除いてボタンを支援技術から参照できる状態にし、aria-label に Close を指定します。

```vue annotate="add:2"
<template>
  <button aria-label="Close" @click="close">Close</button>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_aria_hidden_on_focusable.rs#L19) · [全ルール](all.md)

### `a11y/no-autofocus`

意図せずフォーカスを移動させる autofocus を検出します。

[悪い例](#a11y-no-autofocus-bad) · [良い例](#a11y-no-autofocus-good)

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

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_autofocus.rs#L19) · [全ルール](all.md)

### `a11y/no-distracting-elements`

marquee や blink などの注意をそらす要素を検出します。

[悪い例](#a11y-no-distracting-elements-bad) · [良い例](#a11y-no-distracting-elements-good)

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

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_distracting_elements.rs#L16) · [全ルール](all.md)

### `a11y/no-i-for-icon`

アイコン用の i 要素を検出し、意味に合う要素を勧めます。

[悪い例](#a11y-no-i-for-icon-bad) · [良い例](#a11y-no-i-for-icon-good)

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

アイコンの表示に i 要素を使っています。i が持つテキスト上の意味は、アイコンだけのボタンが行う操作を説明するものではありません。

```vue annotate="remove:3"
<template>
  <button>
    <i class="material-icons">delete</i>
  </button>
</template>
```

<span id="a11y-no-i-for-icon-good"></span>

**良い**

装飾用の span に aria-hidden を指定してアイコンを支援技術から隠し、別の `Delete item` の文字でボタンの操作を説明します。

```vue annotate="add:3,4"
<template>
  <button>
    <span class="material-icons" aria-hidden="true">delete</span>
    <span class="sr-only">Delete item</span>
  </button>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_i_for_icon.rs#L35) · [全ルール](all.md)

### `a11y/no-redundant-roles`

要素本来の意味と重複する ARIA role を検出します。

[悪い例](#a11y-no-redundant-roles-bad) · [良い例](#a11y-no-redundant-roles-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: 対応している診断で利用できます  
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

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_redundant_roles/report.rs#L31) · [全ルール](all.md)

### `a11y/no-refer-to-non-existent-id`

文書内に存在しない ID への参照を検出します。

[悪い例](#a11y-no-refer-to-non-existent-id-bad) · [良い例](#a11y-no-refer-to-non-existent-id-good)

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

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_refer_to_non_existent_id.rs#L37) · [全ルール](all.md)
