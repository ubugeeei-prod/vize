---
title: "vue/attribute-hyphenation"
description: "规范自定义组件的属性命名风格。"
---

# `vue/attribute-hyphenation`

规范自定义组件的属性命名风格。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

组件属性使用了驼峰形式 firstName。

```vue annotate="remove:2"
<template>
<UserCard firstName="Ada" />
</template>
```

## 正确示例

first-name 符合配置要求的连字符组件属性命名规范。

```vue annotate="add:2"
<template>
<UserCard first-name="Ada" />
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
        "vue/attribute-hyphenation": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [规则选项](/rules/options.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_hyphenation.rs#L32)
