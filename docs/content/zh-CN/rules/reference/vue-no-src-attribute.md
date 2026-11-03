---
title: "vue/no-src-attribute"
description: "不建议在 SFC 块上使用 src 属性。"
---

# `vue/no-src-attribute`

不建议在 SFC 块上使用 src 属性。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

SFC 块通过 src 文件提供模板、脚本和样式内容。

```vue annotate="remove:1,2,3"
<template src="./template.html"></template>
<script src="./script.ts"></script>
<style src="./style.css"></style>
```

## 正确示例

每个 SFC 块都包含自身内容，不使用外部 src 属性。

```vue annotate="add:1,2,3,4,5,6,7,8,9,10,11,12,13"
<template>
  <p>Hello</p>
</template>

<script setup lang="ts">
const label = "Hello";
</script>

<style scoped>
p {
  color: red;
}
</style>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/no-src-attribute": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_src_attribute.rs#L16)
