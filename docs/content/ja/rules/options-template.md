---
title: テンプレートのルールのオプション
---

<!-- Reviewed translation; source: rules/options-template.md -->

# テンプレートのルールのオプション

これらのオプションは [eslint-plugin-vue](https://eslint.vuejs.org/rules/) と同じ名前と既定値を
使います。オプションだけではルールは有効になりません。後から一致する設定はそのルールの
オプション全体を置き換え、`{}` で既定値に戻します。未定義のフィールドや型の合わない値は拒否します。

## `vue/no-duplicate-attributes`

`allowCoexistClass` と `allowCoexistStyle` はどちらも既定で `true` です。`class` と `:class`、
`style` と `:style` は同じ要素に並べられます。どちらかを `false` にすると、2つ目の書き方を
重複として報告します。静的な `class` 属性が2つある場合は常に重複です。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    rules: { "vue/no-duplicate-attributes": "error" },
    ruleOptions: { "vue/no-duplicate-attributes": { allowCoexistClass: false } },
  },
});
```

この設定で報告される例:

```vue
<template>
  <div class="card" :class="{ active }" />
</template>
```

この設定に沿った例:

```vue
<template>
  <div :class="['card', { active }]" />
</template>
```

## `vue/no-multiple-template-root`

`disallowComments` の既定は `false` です。`true` にすると、テンプレートのルートに直接置かれ、
どのルート要素にも含まれないコメントも報告します。ルート要素の中のコメントは許可されます。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    rules: { "vue/no-multiple-template-root": "error" },
    ruleOptions: { "vue/no-multiple-template-root": { disallowComments: true } },
  },
});
```

この設定で報告される例:

```vue
<template>
  <!-- The card -->
  <div class="card" />
</template>
```

この設定に沿った例:

```vue
<template>
  <div class="card">
    <!-- The card -->
  </div>
</template>
```

## `vue/no-lone-template`

`ignoreAccessible` の既定は `false` です。`true` にすると、`id` か `ref` を持つ入れ子の
`<template>` は報告しません。静的な属性と、静的な引数でバインドした属性が対象です。
コードから参照できるためです。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    rules: { "vue/no-lone-template": "warn" },
    ruleOptions: { "vue/no-lone-template": { ignoreAccessible: true } },
  },
});
```

この設定で報告される例:

```vue
<template>
  <div>
    <template><span>Row</span></template>
  </div>
</template>
```

この設定に沿った例:

```vue
<template>
  <div>
    <template ref="row"><span>Row</span></template>
  </div>
</template>
```

## `vue/no-static-inline-styles`

`allowBinding` の既定は `false` で、静的な `style` 属性に加えて `:style` バインディング内の
静的な値も報告します。`true` にすると、`style` 属性だけを報告します。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    rules: { "vue/no-static-inline-styles": "warn" },
    ruleOptions: { "vue/no-static-inline-styles": { allowBinding: true } },
  },
});
```

この設定で報告される例:

```vue
<template>
  <div style="color: red" />
</template>
```

この設定に沿った例:

```vue
<template>
  <div :style="{ color: 'red' }" />
</template>
```

## `vue/no-unused-components`

`ignoreWhenBindingPresent` の既定は `true` です。`<component :is="current">` のように `is` を
文字列リテラル以外にバインドするテンプレートは、登録済みのどのコンポーネントも描画できるため、
そのファイルでは何も報告しません。`false` にすると、そのバインディングがない場合と同じように登録を
検査します。HTML の修復でパースできなかったテンプレートは、引き続きすべての登録を使う可能性がある
ものとして扱います。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    rules: { "vue/no-unused-components": "warn" },
    ruleOptions: { "vue/no-unused-components": { ignoreWhenBindingPresent: false } },
  },
});
```

この設定で報告される例:

```vue
<script>
import Card from "./Card.vue";
import Panel from "./Panel.vue";

export default { components: { Card, Panel }, data: () => ({ current: "Card" }) };
</script>

<template>
  <component :is="current" />
</template>
```

この設定に沿った例:

```vue
<script>
import Card from "./Card.vue";

export default { components: { Card } };
</script>

<template>
  <Card />
</template>
```

## `vue/no-v-html`

`ignorePattern` に既定値はありません。式が正規表現に一致する `v-html` は報告しません。
識別子はその名前で、それ以外の式は外側の括弧を除いたソースの文字列で照合します。パターンは
Rust の正規表現の構文で、upstream が付ける `u` フラグと同じく Unicode に対応します。空の
パターンとコンパイルできないパターンは何も除外しません。対象は `v-html` だけで、`innerHTML` と
`outerHTML` のバインディングは引き続き報告します。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    rules: { "vue/no-v-html": "warn" },
    ruleOptions: { "vue/no-v-html": { ignorePattern: "^sanitized" } },
  },
});
```

この設定で報告される例:

```vue
<template>
  <div v-html="article.body" />
</template>
```

この設定に沿った例:

```vue
<template>
  <div v-html="sanitizedBody" />
</template>
```
