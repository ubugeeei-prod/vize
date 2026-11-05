---
title: "vue/no-child-content"
description: "禁止在使用 v-html 或 v-text 时提供子内容。"
---

# `vue/no-child-content`

禁止在使用 v-html 或 v-text 时提供子内容。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

v-text 会替换段落内容，因此编写的后备文本无法在该指令生效后保留。

```vue annotate="remove:2"
<template>
  <p v-text="message">Fallback text</p>
</template>
```

## 正确示例

移除子文本后，v-text 成为段落内容的唯一来源。

```vue annotate="add:2"
<template>
  <p v-text="message" />
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
        "vue/no-child-content": "error"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_child_content.rs#L30)
