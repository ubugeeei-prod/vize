---
title: "vue/no-duplicate-attributes"
description: "禁止在同一元素上重复声明属性。"
---

# `vue/no-duplicate-attributes`

禁止在同一元素上重复声明属性。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

同一个按钮声明了两次 class，而不是将它们合并为一个 class 值。

```vue annotate="remove:2"
<template>
  <button class="primary" class="large">Save</button>
</template>
```

## 正确示例

两个类名都放在同一个 class 属性中。

```vue annotate="add:2"
<template>
  <button class="primary large">Save</button>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-duplicate-attributes": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_duplicate_attributes.rs#L31)
