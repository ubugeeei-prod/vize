---
title: "vue/no-deprecated-router-link-tag-prop"
description: "禁止在 &lt;router-link&gt; 上使用 `tag` prop。"
---

# `vue/no-deprecated-router-link-tag-prop`

禁止在 &lt;router-link&gt; 上使用 `tag` prop。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

RouterLink 使用了已移除的 tag prop 来指定 button 元素。

```vue annotate="remove:2"
<template>
  <router-link to="/home" tag="button">Home</router-link>
</template>
```

## 正确示例

插槽将 navigate 提供给显式编写的 button。

```vue annotate="add:2,3,4"
<template>
  <router-link to="/home" v-slot="{ navigate }">
    <button @click="navigate">Home</button>
  </router-link>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-deprecated-router-link-tag-prop": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_router_link_tag_prop.rs#L38)
