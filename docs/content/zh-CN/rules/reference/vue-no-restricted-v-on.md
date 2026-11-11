---
title: "vue/no-restricted-v-on"
description: "禁止使用配置的 v-on 参数。"
---

# `vue/no-restricted-v-on`

禁止使用配置的 v-on 参数。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

按钮监听被禁止的 legacy-click。

```vue annotate="remove:2"
<template>
  <button type="button" @legacy-click="save">x</button>
</template>
```

## 正确示例

按钮监听 click。

```vue annotate="add:2"
<template>
  <button type="button" @click="save">x</button>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-v-on": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-v-on": [
        "legacy-click"
      ]
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/restricted_v_on.rs#L13)
