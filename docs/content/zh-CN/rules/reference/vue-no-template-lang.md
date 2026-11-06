---
title: "vue/no-template-lang"
description: "不建议在 template 块上使用 lang 属性。"
---

# `vue/no-template-lang`

不建议在 template 块上使用 lang 属性。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

模板通过 lang 选择 Pug。这是只使用 HTML 的预期规范；当前 SFC 路径不会针对该目录 ID 产生诊断。

```vue annotate="remove:1,2"
<template lang="pug">
p Notice
</template>
```

## 正确示例

普通 HTML 模板省略 lang，直接使用段落。这展示了规范，并未声称当前 SFC 会报告问题。

```vue annotate="add:1,2"
<template>
<p>Notice</p>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-template-lang": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_lang.rs#L11)
