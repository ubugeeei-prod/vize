---
title: "vue/no-script-non-standard-lang"
description: "不建议使用非标准的 script lang 值。"
---

# `vue/no-script-non-standard-lang`

不建议使用非标准的 script lang 值。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 尚未在 SFC lint 中实现 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

> [!WARNING]
> 当前支持情况：`no-sfc-finding`。 Vize 接受在配置中使用这个规则 ID，但 SFC lint 目前还不会报告它。正确示例展示的是预期的写法。

此目录条目目前不会通过 SFC lint 报告该规则特有的问题。Bad/Good 对照描述的是预期规范，并非可执行的诊断。启用此 ID 也不会补上缺失的 SFC 检查。

## 错误示例

脚本在 lang=coffee 下使用 CoffeeScript 语法。当前 SFC 路径不会针对该语言报告此目录规则。

```vue annotate="remove:1,2"
<script lang="coffee">
count = 0
</script>
<template><p>Notice</p></template>
```

## 正确示例

脚本在 lang=ts 下使用普通 TypeScript 声明，展示预期的语言规范。

```vue annotate="add:1,2"
<script lang="ts">
const count = 0;
</script>
<template><p>Notice</p></template>
```

## 可配置的 ID（当前没有 SFC 诊断）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-script-non-standard-lang": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_script_non_standard_lang.rs#L44)
