---
title: "vue/require-default-export"
description: "要求组件的 script 作为默认导出。"
---

# `vue/require-default-export`

要求组件的 script 作为默认导出。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

组件 script 没有默认导出。

```vue
<script lang="ts">
const label = "a";
</script>
<template>
  <p>label</p>
</template>
```

## 正确示例

组件是 script 的默认导出。

```vue annotate="add:3"
<script lang="ts">
const label = "a";
export default { name: "Label" };
</script>
<template>
  <p>label</p>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/require-default-export": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_default_export.rs#L14)
