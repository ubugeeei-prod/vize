---
title: Vue ルールのオプション
---
<!-- Reviewed translation; source: rules/options-vue.md -->

<span id="vue-rule-options"></span>

# Vue ルールのオプション

Vue のルールで、コンポーネント名の表記、props の変更範囲、SFC ブロックの並び順、
自己終了タグ、イベント名、属性名を設定する例を紹介します。
設定できる全オプションの一覧、未定義のフィールドの扱い、スコープごとのオプションの置き換えは、
[ルールのオプション](./options.md)を参照してください。

<span id="vue-valid-v-slot"></span>

## `vue/valid-v-slot`：スロット名の修飾子

既定では、静的なスロット名と動的なスロット名のどちらも、
`#item.memo` や `#[name].memo` のような修飾子を許可しません。
Vuetify のデータテーブルのように、コンポーネントの API でドットを含むスロット名を使う場合は、
`allowModifiers` を有効にしてください。

```ts
import { defineConfig } from "vize/config";

export default defineConfig({
  linter: {
    rules: { "vue/valid-v-slot": "error" },
    ruleOptions: {
      "vue/valid-v-slot": { allowModifiers: true },
    },
  },
});
```

この設定で許可される例：

```vue
<FancyPanel>
  <template #item.memo="{ item }">{{ item.memo }}</template>
  <template #[name].memo="slot">{{ slot }}</template>
</FancyPanel>
```

引数を持たない `v-slot.memo` は、どちらの設定でも無効です。
`#[slot.name]="slot"` のような動的な名前も無効です。
スロット名を評価する時点では、そのスロットのパラメーターがまだ存在しないためです。
このオプションが変えるのは修飾子の許可方針だけです。スロットの不正な使い方は、引き続き報告します。
ルート設定が修飾子を許可していても、一致する設定で `allowModifiers: false` を明示すれば、
既定の禁止方針に戻ります。

<span id="vue-component-name-in-template-casing"></span>

## `vue/component-name-in-template-casing`：コンポーネントタグの命名規則

テンプレートのコンポーネントタグを PascalCase または kebab-case に統一するためのオプションです。
既定では、`<script setup>` の宣言と `components` に直接登録されたコンポーネント名を検査します。
未登録のタグや、スクリプトのないテンプレートは対象外です。
`registeredComponentsOnly: false` を指定すると、すべてのカスタムタグを検査します。
グローバル登録したコンポーネントを明示するには、`globals: ["PrimaryButton"]` を使います。
グローバル名は文字列で指定します。正規表現と ESLint の `ignores` オプションは未対応です。
通常の HTML、SVG、MathML の要素、`slot` と動的な `component` タグは対象外です。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "rules": {
        "vue/component-name-in-template-casing": "warn"
      },
      "ruleOptions": {
        "vue/component-name-in-template-casing": {
          "casing": "kebab-case"
        }
      }
    }
  }
});
```

上の設定で報告される例：

```vue
<script setup>
import PrimaryButton from "./PrimaryButton.vue";
</script>
<template>
  <PrimaryButton />
</template>
```

設定に沿った例：

```vue
<script setup>
import PrimaryButton from "./PrimaryButton.vue";
</script>
<template>
  <primary-button />
</template>
```

<span id="vue-no-mutating-props"></span>

## `vue/no-mutating-props`：props の変更を許す範囲

移行中のプロジェクトで、入れ子の props の変更は一時的に許可し、prop 自体の再代入は
引き続き禁止したいときに、`shallowOnly` を使います。
既定の `false` では、直接の変更も入れ子の変更も報告します。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "rules": {
        "vue/no-mutating-props": "error"
      },
      "ruleOptions": {
        "vue/no-mutating-props": {
          "shallowOnly": true
        }
      }
    }
  }
});
```

上の設定でも報告される例：

```vue
<script setup>
const props = defineProps<{ count: number; settings: { dense: boolean } }>();

props.count = 2;
</script>
```

上の設定で許可される例：

```vue
<script setup>
const props = defineProps<{ settings: { dense: boolean } }>();

props.settings.dense = true;
</script>
```

<span id="vue-sfc-element-order"></span>

## `vue/sfc-element-order`：SFC ブロックの並び順

SFC ブロックの並び順を統一するためのオプションです。
1 つの文字列は、順序の 1 段階を表します。入れ子の配列は、その段階に
配列内のどのブロックセレクターが現れてもよいことを表します。
組み込みのセレクターは `script`、`script:not([setup])`、`script[setup]`、`template`、`style` です。
それ以外の空でない文字列は、カスタムブロック名に一致します。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "rules": {
        "vue/sfc-element-order": "warn"
      },
      "ruleOptions": {
        "vue/sfc-element-order": {
          "order": [
            "template",
            "script:not([setup])",
            "script[setup]",
            "i18n",
            "style"
          ]
        }
      }
    }
  }
});
```

上の設定で報告される例：

```vue
<style scoped></style>
<template></template>
<script setup></script>
```

設定に沿った例：

```vue
<template></template>
<script setup></script>
<style scoped></style>
```

<span id="vue-html-self-closing"></span>

## `vue/html-self-closing`：空の要素の自己終了形式

空の要素を自己終了タグで書くかどうかを、要素の種類ごとに指定します。
`always` は空の要素に自己終了形式を要求し、`never` は自己終了タグを禁止します。
`any` はどちらの形式も許可します。入れ子の設定項目を省略した場合は、Vize の既定値を使います。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "rules": {
        "vue/html-self-closing": "warn"
      },
      "ruleOptions": {
        "vue/html-self-closing": {
          "html": {
            "void": "always",
            "normal": "never",
            "component": "always"
          },
          "svg": "always",
          "math": "always"
        }
      }
    }
  }
});
```

上の設定で報告される例：

```vue
<template>
  <div />
  <PrimaryButton></PrimaryButton>
</template>
```

設定に沿った例：

```vue
<template>
  <div></div>
  <PrimaryButton />
</template>
```

<span id="vue-v-on-event-hyphenation"></span>

## `vue/v-on-event-hyphenation`：コンポーネントのイベント名

コンポーネントのカスタムイベントに付ける、静的なリスナー名を指定するためのオプションです。
`always` は `@saveItem` のような camelCase のリスナー引数を報告し、
`never` はハイフン区切りのリスナー引数を報告します。
通常の HTML 要素のイベント、オブジェクト形式の指定、動的な引数は対象外です。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "rules": {
        "vue/v-on-event-hyphenation": "warn"
      },
      "ruleOptions": {
        "vue/v-on-event-hyphenation": "always"
      }
    }
  }
});
```

上の設定で報告される例：

```vue
<template>
  <PrimaryButton @saveItem="save" />
</template>
```

設定に沿った例：

```vue
<template>
  <PrimaryButton @save-item="save" />
  <button @saveItem="save" />
</template>
```

<span id="vue-attribute-hyphenation"></span>

## `vue/attribute-hyphenation`：コンポーネントに渡す属性名

カスタムコンポーネントに渡す、静的な props の属性名を指定するためのオプションです。
`always` は camelCase で書かれた属性を報告し、`never` はハイフン区切りで書かれた属性を報告します。
通常の要素の属性、`aria-*`、`data-*`、SVG の大文字と小文字が混在する属性、
動的な引数、属性として解析されるディレクティブの省略記法は対象外です。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "rules": {
        "vue/attribute-hyphenation": "warn"
      },
      "ruleOptions": {
        "vue/attribute-hyphenation": "never"
      }
    }
  }
});
```

上の設定で報告される例：

```vue
<template>
  <UserCard user-name="Ada" />
</template>
```

設定に沿った例：

```vue
<template>
  <UserCard userName="Ada" aria-label="Ada Lovelace" />
</template>
```
