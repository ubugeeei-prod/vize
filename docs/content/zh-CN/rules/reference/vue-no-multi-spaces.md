---
title: "vue/no-multi-spaces"
description: "禁止连续多个空格。"
---

# `vue/no-multi-spaces`

禁止连续多个空格。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

属性之间，或元素名称与第一个属性之间，使用了两个空格。

```vue annotate="remove:2,3"
<template>
  <div  class="panel"></div>
  <div class="panel"  id="main"></div>
</template>
```

## 正确示例

同样的属性之间使用单个空格。

```vue annotate="add:2,3"
<template>
  <div class="panel"></div>
  <div class="panel" id="main"></div>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-multi-spaces": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multi_spaces.rs#L26)
