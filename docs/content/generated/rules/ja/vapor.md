---
title: "Vapor ルール"
---

# Vapor ルール

このページに、各ルールの目的・前提・設定・完全な悪い例と良い例をまとめています。変更箇所は行の色で示し、コピーしたコードには完全なソースを保持します。各例に現在の対応範囲も明記しています。


| ルール | 例 | 目的 |
| --- | --- | --- |
| [`script/no-get-current-instance`](#script-no-get-current-instance) | [悪い](#script-no-get-current-instance-bad) · [良い](#script-no-get-current-instance-good) | Vapor で null を返す getCurrentInstance() を検出します。 |
| [`script/no-next-tick`](#script-no-next-tick) | [悪い](#script-no-next-tick-bad) · [良い](#script-no-next-tick-good) | Vapor 向けコンポーネントの nextTick() 使用を検出します。 |
| [`script/no-options-api`](#script-no-options-api) | [悪い](#script-no-options-api-bad) · [良い](#script-no-options-api-good) | Vapor で Options API を使用する箇所を検出します。 |
| [`vapor/no-inline-template`](#vapor-no-inline-template) | [悪い](#vapor-no-inline-template-bad) · [良い](#vapor-no-inline-template-good) | Vapor で削除済みの inline-template 属性を検出します。 |
| [`vapor/no-vue-lifecycle-events`](#vapor-no-vue-lifecycle-events) | [悪い](#vapor-no-vue-lifecycle-events-bad) · [良い](#vapor-no-vue-lifecycle-events-good) | Vapor が対応しない要素の @vue:* lifecycle event を検出します。 |
| [`vapor/prefer-static-class`](#vapor-prefer-static-class) | [悪い](#vapor-prefer-static-class-bad) · [良い](#vapor-prefer-static-class-good) | 文字列リテラルの :class を静的 class に置き換えます。 |
| [`vapor/require-vapor-attribute`](#vapor-require-vapor-attribute) | [悪い](#vapor-require-vapor-attribute-bad) · [良い](#vapor-require-vapor-attribute-good) | Vapor 向けの script setup に vapor 属性を付ける方針を適用します。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)

### `script/no-get-current-instance`

Vapor で null を返す getCurrentInstance() を検出します。

[悪い例](#script-no-get-current-instance-bad) · [良い例](#script-no-get-current-instance-good)

既定の重大度: `error`  
プリセット: `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vapor を想定した script 検査。明示的に有効にすると通常の script でも同じ禁止を適用します。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-get-current-instance": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-get-current-instance-bad"></span>

**悪い**

Vapor を指定した setup が `getCurrentInstance` をインポートして呼び出し、Vapor 向けコンポーネントでこのルールが禁止するインスタンス API に依存しています。

```vue annotate="remove:2,3"
<script setup lang="ts" vapor>
import { getCurrentInstance } from "vue";
const instance = getCurrentInstance();
</script>
```

<span id="script-no-get-current-instance-good"></span>

**良い**

`inject("app-config")` で明示的に提供された設定を受け取り、`getCurrentInstance` のインポートも呼び出しも使いません。

```vue annotate="add:2,3"
<script setup lang="ts" vapor>
import { inject } from "vue";
const appConfig = inject("app-config");
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_get_current_instance.rs#L38) · [全ルール](all.md)

### `script/no-next-tick`

Vapor 向けコンポーネントの nextTick() 使用を検出します。

[悪い例](#script-no-next-tick-bad) · [良い例](#script-no-next-tick-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vapor を想定した script 検査。明示的に有効にすると通常の script でも同じ禁止を適用します。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-next-tick": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-next-tick-bad"></span>

**悪い**

Vapor 向けコンポーネントが `nextTick` をインポートして await し、この移行ルールが拒否する DOM 更新待ちの依存を作っています。

```vue annotate="remove:2,3"
<script setup lang="ts" vapor>
import { nextTick } from "vue";
await nextTick();
</script>
```

<span id="script-no-next-tick-good"></span>

**良い**

`useTemplateRef` で input を取得し、`onMounted` でフォーカスします。例の `nextTick` への依存を、明示的なマウント時の処理に置き換えます。

```vue annotate="add:2,3,4,6"
<script setup lang="ts" vapor>
import { onMounted, useTemplateRef } from "vue";
const input = useTemplateRef<HTMLInputElement>("input");
onMounted(() => { input.value?.focus(); });
</script>
<template><input ref="input"></template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_next_tick.rs#L40) · [全ルール](all.md)

### `script/no-options-api`

Vapor で Options API を使用する箇所を検出します。

[悪い例](#script-no-options-api-bad) · [良い例](#script-no-options-api-good)

既定の重大度: `error`  
プリセット: `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vapor を想定した script 検査。明示的に有効にすると通常の script でも同じ禁止を適用します。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-options-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-options-api-bad"></span>

**悪い**

default export のオブジェクトに Options API の `data()` を宣言しており、このルールが禁止するコンポーネントオプションの形式です。

```vue annotate="remove:1,2,3,4,5,6"
<script lang="ts">
export default {
  data() {
    return { count: 0 };
  },
};
</script>
```

<span id="script-no-options-api-good"></span>

**良い**

状態を Vapor の `<script setup>` 内の Composition API `ref` に移し、Options API のオブジェクトと `data` オプションを取り除きます。

```vue annotate="add:1,2"
<script setup lang="ts" vapor>
const count = ref(0);
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_options_api.rs#L43) · [全ルール](all.md)

### `vapor/no-inline-template`

Vapor で削除済みの inline-template 属性を検出します。

[悪い例](#vapor-no-inline-template-bad) · [良い例](#vapor-no-inline-template-good)

既定の重大度: `error`  
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
        "vapor/no-inline-template": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vapor-no-inline-template-bad"></span>

**悪い**

LegacyCard の内部のマークアップに inline-template を使っています。

```vue annotate="remove:2,3"
<template>
  <LegacyCard inline-template>
    <p>Profile</p>
  </LegacyCard>
</template>
```

<span id="vapor-no-inline-template-good"></span>

**良い**

同じマークアップを default slot として渡します。

```vue annotate="add:2,3,4,5"
<template>
  <LegacyCard>
    <template #default>
      <p>Profile</p>
    </template>
  </LegacyCard>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/no_inline_template.rs#L31) · [全ルール](all.md)

### `vapor/no-vue-lifecycle-events`

Vapor が対応しない要素の @vue:* lifecycle event を検出します。

[悪い例](#vapor-no-vue-lifecycle-events-bad) · [良い例](#vapor-no-vue-lifecycle-events-good)

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
        "vapor/no-vue-lifecycle-events": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vapor-no-vue-lifecycle-events-bad"></span>

**悪い**

input にテンプレートの @vue:mounted を指定しています。

```vue annotate="remove:2"
<template>
  <input @vue:mounted="focusInput" />
</template>
```

<span id="vapor-no-vue-lifecycle-events-good"></span>

**良い**

script の onMounted からテンプレートの ref を参照し、input にフォーカスします。

```vue annotate="add:1,2,3,4,5,6,7,8,10"
<script setup lang="ts" vapor>
const input = useTemplateRef<HTMLInputElement>("input");

onMounted(() => {
  input.value?.focus();
});
</script>

<template>
  <input ref="input" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vapor/no_vue_lifecycle_events.rs#L34) · [全ルール](all.md)

### `vapor/prefer-static-class`

文字列リテラルの :class を静的 class に置き換えます。

[悪い例](#vapor-prefer-static-class-bad) · [良い例](#vapor-prefer-static-class-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
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
        "vapor/prefer-static-class": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vapor-prefer-static-class-bad"></span>

**悪い**

変化しないクラスの文字列をバインディングで評価しています。

```vue annotate="remove:2"
<template>
  <section :class="'panel panel-primary'">Profile</section>
</template>
```

<span id="vapor-prefer-static-class-good"></span>

**良い**

同じ panel のクラスを静的な class 属性に指定します。

```vue annotate="add:2"
<template>
  <section class="panel panel-primary">Profile</section>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/prefer_static_class.rs#L31) · [全ルール](all.md)

### `vapor/require-vapor-attribute`

Vapor 向けの script setup に vapor 属性を付ける方針を適用します。

[悪い例](#vapor-require-vapor-attribute-bad) · [良い例](#vapor-require-vapor-attribute-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: SFC lint では未対応  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

現在の対応: `no-sfc-finding`

このルールは callback が空の placeholder です。vapor 属性は Vapor でのコンパイルを選択するものですが、現在の linter は属性がないことをこの ID では検出しません。

**設定できる ID（現在の SFC 検出なし）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vapor/require-vapor-attribute": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vapor-require-vapor-attribute-bad"></span>

**悪い**

script setup に Vapor の指定がありません。これは規約を示す例で、現在の空の callback は診断を生成しません。

```vue annotate="remove:1"
<script setup>
const count = 0;
</script>
<template><p>{{ count }}</p></template>
```

<span id="vapor-require-vapor-attribute-good"></span>

**良い**

vapor を追加して Vapor のコンパイルを選択します。修正方針を示す例であり、現在の linter がこのルールを検出するという意味ではありません。

```vue annotate="add:1"
<script setup vapor>
const count = 0;
</script>
<template><p>{{ count }}</p></template>
```

良い例は意図する規約を示します。現在の SFC の処理は、どちらの例でもこのルール固有の診断を生成しません。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/require_vapor_attribute.rs#L17) · [全ルール](all.md)
