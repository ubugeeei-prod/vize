---
title: "コンポーネントと props のルール"
---

<span id="vue-rules-components-and-props"></span>

# コンポーネントと props のルール

このルール群では、コンポーネントの名前・登録・使い方と、props の変更や未使用の宣言を確認します。
コンポーネントを追加したり、親子間で渡すデータや動的コンポーネントの切り替えを変更したりするときに確認してください。

たとえば、[props を直接変更しない例](./reference/vue-no-mutating-props.md)では、親から受け取った値を書き換える代わりに、更新をイベントで親へ通知する形を比較できます。
掲載された設定でルールを有効にし、`vp run lint` を実行してください。
[Vue の全ルール](./vue.md)では、テンプレートや SFC の関連ルールも確認できます。

このページに、各ルールの目的・前提・設定と、省略のない悪い例・良い例をまとめています。変更箇所は行の色で示します。コードをコピーすると、ソースコード全体を取得できます。各例に現在の対応範囲も明記しています。


| ルール | 例 | 目的 |
| --- | --- | --- |
| [`vue/component-definition-name-casing`](#vue-component-definition-name-casing) | [悪い](#vue-component-definition-name-casing-bad) · [良い](#vue-component-definition-name-casing-good) | コンポーネント定義名を PascalCase または kebab-case に揃えます。 |
| [`vue/component-name-in-template-casing`](#vue-component-name-in-template-casing) | [悪い](#vue-component-name-in-template-casing-bad) · [良い](#vue-component-name-in-template-casing-good) | テンプレート内のコンポーネント名を指定した形式に揃えます。 |
| [`vue/multi-word-component-names`](#vue-multi-word-component-names) | [悪い](#vue-multi-word-component-names-bad) · [良い](#vue-multi-word-component-names-good) | コンポーネント名を複数の単語で構成します。 |
| [`vue/no-mutating-props`](#vue-no-mutating-props) | [悪い](#vue-no-mutating-props-bad) · [良い](#vue-no-mutating-props-good) | 親から受け取った prop を子コンポーネントで変更する箇所を検出します。 |
| [`vue/no-non-component-keep-alive-child`](#vue-no-non-component-keep-alive-child) | [悪い](#vue-no-non-component-keep-alive-child-bad) · [良い](#vue-no-non-component-keep-alive-child-good) | KeepAlive の直下に通常の HTML 要素を置く箇所を検出します。 |
| [`vue/no-reserved-component-names`](#vue-no-reserved-component-names) | [悪い](#vue-no-reserved-component-names-bad) · [良い](#vue-no-reserved-component-names-good) | 予約済みのコンポーネント名を検出します。 |
| [`vue/no-unused-components`](#vue-no-unused-components) | [悪い](#vue-no-unused-components-bad) · [良い](#vue-no-unused-components-good) | 登録しているのにテンプレートで使わないコンポーネントを検出します。 |
| [`vue/no-unused-properties`](#vue-no-unused-properties) | [悪い](#vue-no-unused-properties-bad) · [良い](#vue-no-unused-properties-good) | defineProps に宣言しているのに使わない prop を検出します。 |
| [`vue/require-component-is`](#vue-require-component-is) | [悪い](#vue-require-component-is-bad) · [良い](#vue-require-component-is-good) | 動的 component 要素に :is を指定します。 |
| [`vue/require-component-registration`](#vue-require-component-registration) | [悪い](#vue-require-component-registration-bad) · [良い](#vue-require-component-registration-good) | 使用するコンポーネントを import または登録します。 |

[全ルール](./all.md) · [ルールのオプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)

### `vue/component-definition-name-casing`

コンポーネント定義名を PascalCase または kebab-case に揃えます。

[悪い例](#vue-component-definition-name-casing-bad) · [良い例](#vue-component-definition-name-casing-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

対象はファイル名です。PascalCase と kebab-case は許可され、混在する形式は検出されます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/component-definition-name-casing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-component-definition-name-casing-bad"></span>

**悪い**

ファイル名 myComponent.vue が先頭の小文字と途中の大文字を混在させ、PascalCase / kebab-case のどちらにもなっていません。

`myComponent.vue`

```vue
<template><p>Content</p></template>
```

<span id="vue-component-definition-name-casing-good"></span>

**良い**

ファイル名を PascalCase の MyComponent.vue に変更します。テンプレートの内容は同じです。

`MyComponent.vue`

```vue
<template><p>Content</p></template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/component_definition_name_casing.rs#L36) · [全ルール](all.md)

### `vue/component-name-in-template-casing`

テンプレート内のコンポーネント名を指定した形式に揃えます。

[悪い例](#vue-component-name-in-template-casing-bad) · [良い例](#vue-component-name-in-template-casing-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: 対応している診断で利用できます  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: [型付きオプションと既定値](options.md)を参照してください。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/component-name-in-template-casing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-component-name-in-template-casing-bad"></span>

**悪い**

PascalCase の方針に対し、コンポーネントを kebab-case と camelCase で記述しています。

```vue annotate="remove:5,6"
<script setup>
import MyComponent from "./MyComponent.vue";
</script>
<template>
  <my-component />
  <myComponent />
</template>
```

<span id="vue-component-name-in-template-casing-good"></span>

**良い**

MyComponent を PascalCase で記述します。標準の slot は小文字のままです。

```vue annotate="add:5,6,7"
<script setup>
import MyComponent from "./MyComponent.vue";
</script>
<template>
  <MyComponent />
  <RouterView />
  <slot />
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/component_name_in_template_casing.rs#L31) · [全ルール](all.md)

### `vue/multi-word-component-names`

コンポーネント名を複数の単語で構成します。

[悪い例](#vue-multi-word-component-names-bad) · [良い例](#vue-multi-word-component-names-good)

既定の重大度: `error`  
プリセット: `essential`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

検出対象はファイル名です。同じコンポーネントのファイル名を変更します。子要素のタグ名は対象ではありません。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/multi-word-component-names": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-multi-word-component-names-bad"></span>

**悪い**

Item.vue は一つの語だけのコンポーネント名です。

`Item.vue`

```vue
<template><p>Item</p></template>
```

<span id="vue-multi-word-component-names-good"></span>

**良い**

同じコンポーネントのファイル名を、複数の単語からなる TodoItem.vue に変更します。

`TodoItem.vue`

```vue
<template><p>Item</p></template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/multi_word_component_names.rs#L34) · [全ルール](all.md)

### `vue/no-mutating-props`

親から受け取った prop を子コンポーネントで変更する箇所を検出します。

[悪い例](#vue-no-mutating-props-bad) · [良い例](#vue-no-mutating-props-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: [型付きオプションと既定値](options.md)を参照してください。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-mutating-props": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-mutating-props-bad"></span>

**悪い**

props.count の加算で、親から受け取った値を直接変更しています。

```vue annotate="remove:4"
<script setup lang="ts">
const props = defineProps<{ count: number }>();

props.count++;
</script>
```

<span id="vue-no-mutating-props-good"></span>

**良い**

次の値を update:count で通知し、prop の変更は親が行う形にします。

```vue annotate="add:3,5,6,7"
<script setup lang="ts">
const props = defineProps<{ count: number }>();
const emit = defineEmits<{ "update:count": [value: number] }>();

function increment() {
  emit("update:count", props.count + 1);
}
</script>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_mutating_props.rs#L36) · [全ルール](all.md)

### `vue/no-non-component-keep-alive-child`

KeepAlive の直下に通常の HTML 要素を置く箇所を検出します。

[悪い例](#vue-no-non-component-keep-alive-child-bad) · [良い例](#vue-no-non-component-keep-alive-child-good)

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
        "vue/no-non-component-keep-alive-child": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-non-component-keep-alive-child-bad"></span>

**悪い**

KeepAlive の条件付きの子が UserCard ではなく、標準の div になっています。

```vue annotate="remove:3"
<template>
  <KeepAlive>
    <div v-if="ready">
      <UserCard />
    </div>
  </KeepAlive>
</template>
```

<span id="vue-no-non-component-keep-alive-child-good"></span>

**良い**

最初の例は UserCard を条件付きの子にします。v-show のラッパー要素はこの条件付きの子の検査の対象外を示す例で、標準要素がキャッシュされるという意味ではありません。

```vue annotate="add:3,4,5,6"
<template>
  <KeepAlive>
    <UserCard v-if="ready" />
  </KeepAlive>
  <KeepAlive>
    <div v-show="opened">
      <UserCard />
    </div>
  </KeepAlive>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_non_component_keep_alive_child.rs#L26) · [全ルール](all.md)

### `vue/no-reserved-component-names`

予約済みのコンポーネント名を検出します。

[悪い例](#vue-no-reserved-component-names-bad) · [良い例](#vue-no-reserved-component-names-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "vue/no-reserved-component-names": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-reserved-component-names-bad"></span>

**悪い**

コンポーネント名の button が、標準の HTML 要素名と競合しています。

```vue annotate="remove:1,2,3,4"
<script>
export default {
  name: "button",
};
</script>
```

<span id="vue-no-reserved-component-names-good"></span>

**良い**

標準の button と重複しない、アプリのコンポーネント名 AppButton を指定します。

```vue annotate="add:1,2,4,5,6,7,8,9"
<script setup lang="ts">
defineOptions({ name: "AppButton" });
</script>

<template>
  <Transition>
    <AppButton />
  </Transition>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_reserved_component_names.rs#L45) · [全ルール](all.md)

### `vue/no-unused-components`

登録しているのにテンプレートで使わないコンポーネントを検出します。

[悪い例](#vue-no-unused-components-bad) · [良い例](#vue-no-unused-components-good)

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
        "vue/no-unused-components": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-unused-components-bad"></span>

**悪い**

UserAvatar をコンポーネントとして import していますが、テンプレートで使っていません。

```vue annotate="remove:6"
<script setup lang="ts">
import UserAvatar from "./UserAvatar.vue";
</script>

<template>
  <p>{{ user.name }}</p>
</template>
```

<span id="vue-no-unused-components-good"></span>

**良い**

import した UserAvatar をテンプレートで表示し、user を渡します。

```vue annotate="add:6"
<script setup lang="ts">
import UserAvatar from "./UserAvatar.vue";
</script>

<template>
  <UserAvatar :user="user" />
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_components.rs#L47) · [全ルール](all.md)

### `vue/no-unused-properties`

defineProps に宣言しているのに使わない prop を検出します。

[悪い例](#vue-no-unused-properties-bad) · [良い例](#vue-no-unused-properties-good)

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
        "vue/no-unused-properties": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-unused-properties-bad"></span>

**悪い**

description の prop を宣言していますが、表示には title しか使っていません。

```vue
<script setup lang="ts">
defineProps<{ title: string; description: string }>();
</script>

<template>
  <h1>{{ title }}</h1>
</template>
```

<span id="vue-no-unused-properties-good"></span>

**良い**

宣言した両方の prop をテンプレートで参照します。

```vue annotate="add:7"
<script setup lang="ts">
defineProps<{ title: string; description: string }>();
</script>

<template>
  <h1>{{ title }}</h1>
  <p>{{ description }}</p>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_properties.rs#L94) · [全ルール](all.md)

### `vue/require-component-is`

動的 component 要素に :is を指定します。

[悪い例](#vue-require-component-is-bad) · [良い例](#vue-require-component-is-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "vue/require-component-is": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-require-component-is-bad"></span>

**悪い**

動的な `<component>` に `is` がなく、描画するコンポーネントを選べません。

```vue annotate="remove:2"
<template>
  <component />
</template>
```

<span id="vue-require-component-is-good"></span>

**良い**

`:is="currentComponent"` で描画対象を指定します。対象は実行時に変更できます。

```vue annotate="add:2"
<template>
  <component :is="currentComponent" />
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_component_is.rs#L27) · [全ルール](all.md)

### `vue/require-component-registration`

使用するコンポーネントを import または登録します。

[悪い例](#vue-require-component-registration-bad) · [良い例](#vue-require-component-registration-good)

既定の重大度: `warning`  
プリセット: `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: [型付きオプションと既定値](options.md)を参照してください。

アプリケーションのプラグインや Musea の previewSetup が登録するコンポーネント名を明示します。PascalCase と kebab-case を許可し、正規表現は解釈しません。オプションだけではルールは有効になりません。後の設定は一覧全体を置き換え、空の一覧は継承した名前を消します。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/require-component-registration": "warn"
      },
      "ruleOptions": {
        "vue/require-component-registration": {
          "globals": [
            "MyButton",
            "MyIcon"
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

<span id="vue-require-component-registration-bad"></span>

**悪い**

`MissingWidget` は登録されておらず、設定したグローバルコンポーネント一覧にもありません。

```vue annotate="remove:2"
<template>
<MissingWidget />
</template>
```

<span id="vue-require-component-registration-good"></span>

**良い**

`MyButton` は例の `globals` に含まれます。既知のグローバルコンポーネントを検査対象から除く設定で、import や登録そのものは行いません。

```vue annotate="add:2"
<template>
<MyButton />
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/require_component_registration.rs#L57) · [全ルール](all.md)
