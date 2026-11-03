---
title: "vapor/prefer-static-class"
description: "字符串字面量优先使用静态 class，而不是动态 class 绑定。"
---

# `vapor/prefer-static-class`

字符串字面量优先使用静态 class，而不是动态 class 绑定。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

class 绑定对常量字符串求值，但类名并不会改变。

```vue annotate="remove:2"
<template>
  <section :class="'panel panel-primary'">Profile</section>
</template>
```

## 正确示例

静态 class 属性表达相同的 panel 类名，无需绑定。

```vue annotate="add:2"
<template>
  <section class="panel panel-primary">Profile</section>
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
        "vapor/prefer-static-class": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vapor 规则](../vapor.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/prefer_static_class.rs#L31)
