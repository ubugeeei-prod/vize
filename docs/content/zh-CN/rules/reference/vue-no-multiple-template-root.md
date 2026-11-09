---
title: "vue/no-multiple-template-root"
description: "禁止模板包含多个根节点。"
---

# `vue/no-multiple-template-root`

禁止模板包含多个根节点。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

仅在需要单根节点约定时启用。Vue 3 通常支持片段。

## 错误示例

启用的单根节点规范在模板根部发现了两个同级段落。

```vue annotate="remove:2,3"
<template>
<p>First</p>
<p>Second</p>
</template>
```

## 正确示例

使用 section 将段落包装为一个根节点；仅在需要单根节点约定时启用此规范。

```vue annotate="add:2"
<template>
<section><p>First</p><p>Second</p></section>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-multiple-template-root": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multiple_template_root.rs#L22)
