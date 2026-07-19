---
title: 蒸気のルール
---

<!-- Generated translation; source: rules/vapor.md -->

# 蒸気ルール

これらのルールは、Vapor 指向のコンポーネントとアプリのテンプレート制約をカバーします。構成APIと
スクリプトレベルの Vapor ガイダンスは [タイプとスクリプトのルール](./type-and-script.md) にあります。

## `vapor/no-vue-lifecycle-events`

`@vue:mounted` などの要素ごとのライフサイクル イベントをレポートします。

デフォルトの重大度: `error`
プリセット: `happy-path`、`nuxt`、`opinionated`

悪い：

```vue
<template>
  <input @vue:mounted="focusInput" />
</template>
```

良い：

```vue
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

## `vapor/require-vapor-attribute`

プリセットが Vapor 互換コンポーネントを必要とする場合、`vapor` を `<script setup>` に追加することを提案します。

デフォルトの重大度: `warning`
プリセット: `nuxt`、`opinionated`

悪い：

```vue
<script setup lang="ts">
const count = ref(0);
</script>
```

良い：

```vue
<script setup lang="ts" vapor>
const count = ref(0);
</script>
```

## `vapor/no-inline-template`

非推奨の `inline-template` 属性を報告します。

デフォルトの重大度: `error`
プリセット: `nuxt`、`opinionated`

悪い：

```vue
<template>
  <LegacyCard inline-template>
    <p>Profile</p>
  </LegacyCard>
</template>
```

良い：

```vue
<template>
  <LegacyCard>
    <template #default>
      <p>Profile</p>
    </template>
  </LegacyCard>
</template>
```

## `vapor/prefer-static-class`

値が静的文字列リテラルである動的 `:class` バインディングをレポートします。

デフォルトの重大度: `warning`
プリセット: `nuxt`、`opinionated`

悪い：

```vue
<template>
  <section :class="'panel panel-primary'">Profile</section>
</template>
```

良い：

```vue
<template>
  <section class="panel panel-primary">Profile</section>
</template>
```
