---
title: SSR规则
---

<!-- Generated translation; source: rules/ssr.md -->

# SSR规则

这些规则涵盖了可能破坏服务器渲染或水合的代码和模板模式。它们是
与HTML和Vapor规则分开文档，因为失败模式是服务器/客户端
界限。

## `ssr/no-browser-globals-in-ssr`

报告浏览器专用全局代码，可在SSR期间运行。

默认严重程度：`warning`
预设：`happy-path`，`nuxt`，`opinionated`

缺点：

```vue
<script setup lang="ts">
const width = window.innerWidth;
</script>
```

好：

```vue
<script setup lang="ts">
const width = ref(0);

onMounted(() => {
  width.value = window.innerWidth;
});
</script>
```

像`typeof window === "undefined"`这样的守卫检定被允许，因为直接的`typeof`
标识符表单在服务器渲染时是安全的。字符串、注释和正则表达式文字也是
当名字包含`window`或`document`时，则被忽视。访问成员，例如：
`typeof window.innerWidth`仍然报告，因为它评估浏览器的全球性。

## `ssr/no-hydration-mismatch`

报告非确定性模板值，服务器渲染和客户端可能不同
补充水分。

默认严重程度：`warning`
预设：`happy-path`，`nuxt`，`opinionated`

缺点：

```vue
<template>
  <p>{{ Math.random() }}</p>
</template>
```

好：

```vue
<script setup lang="ts">
const seed = useState("seed", () => "stable");
</script>

<template>
  <p>{{ seed }}</p>
</template>
```
