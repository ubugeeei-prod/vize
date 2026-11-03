---
title: "vue/no-textarea-mustache"
description: "禁止在 `<textarea>` 内使用双花括号插值。"
---

# `vue/no-textarea-mustache`

禁止在 `<textarea>` 内使用双花括号插值。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

textarea 将 message 写在子内容插值中，而没有绑定其值。

```vue annotate="remove:2"
<template>
  <textarea>{{ message }}</textarea>
</template>
```

## 正确示例

v-model 将可编辑的 textarea 值绑定到 message。

```vue annotate="add:2"
<template>
  <textarea v-model="message"></textarea>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/no-textarea-mustache": "error"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_textarea_mustache.rs#L26)
