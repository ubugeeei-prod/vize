---
title: "vue/no-root-v-if"
description: "禁止在模板的唯一根元素上使用 v-if。"
---

# `vue/no-root-v-if`

禁止在模板的唯一根元素上使用 v-if。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

组件根元素本身会在 v-if 控制下出现或消失。

```vue annotate="remove:2"
<template>
  <div v-if="show">content</div>
</template>
```

## 正确示例

稳定的外层 div 始终作为根元素，而内层段落承担可见性条件。

```vue annotate="add:2,3,4"
<template>
  <div>
    <p v-if="show">content</p>
  </div>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-root-v-if": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_root_v_if.rs#L40)
