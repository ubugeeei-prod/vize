---
title: SSRルール
---

<!-- Generated translation; source: rules/ssr.md -->

# SSR ルール

これらのルールは、サーバーのレンダリングやハイドレーションを中断する可能性のあるコードとテンプレートのパターンをカバーします。彼らは
障害モードはサーバー/クライアントであるため、HTML および Vapor ルールとは別に文書化されます。
境界線。

## `ssr/no-browser-globals-in-ssr`

SSR 中に実行できるコード内のブラウザー専用グローバルをレポートします。

デフォルトの重大度: `warning`
プリセット: `happy-path`、`nuxt`、`opinionated`

悪い：

```vue
<script setup lang="ts">
const width = window.innerWidth;
</script>
```

良い：

```vue
<script setup lang="ts">
const width = ref(0);

onMounted(() => {
  width.value = window.innerWidth;
});
</script>
```

`typeof window === "undefined"` などのガード チェックは、直接 `typeof` であるため許可されます。
識別子形式はサーバーレンダリング中は安全です。文字列、コメント、正規表現リテラルも同様です。
`window` や `document` のような名前が含まれている場合は無視されます。次のようなメンバーにアクセスする
`typeof window.innerWidth` はブラウザー グローバルを評価するため、引き続きレポートします。

## `ssr/no-hydration-mismatch`

サーバーレンダーとクライアント間で異なる可能性がある非決定的なテンプレート値をレポートします。
水分補給。

デフォルトの重大度: `warning`
プリセット: `happy-path`、`nuxt`、`opinionated`

悪い：

```vue
<template>
  <p>{{ Math.random() }}</p>
</template>
```

良い：

```vue
<script setup lang="ts">
const seed = useState("seed", () => "stable");
</script>

<template>
  <p>{{ seed }}</p>
</template>
```
