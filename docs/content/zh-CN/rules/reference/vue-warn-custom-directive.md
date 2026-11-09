---
title: "vue/warn-custom-directive"
description: "警告需要注册的自定义指令。"
---

# `vue/warn-custom-directive`

警告需要注册的自定义指令。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

列出全局注册指令的精确名称，不包含 v- 前缀。仍会自动发现 script-setup 中本地的 vName 绑定。选项不会启用规则；后续配置层会替换整个列表，[] 会清除继承的名称。

## 错误示例

`v-focus` 需要项目特定的指令实现，因此被此可选规范标记。

```vue annotate="remove:2"
<template>
<button type="button" v-focus>Focus</button>
</template>
```

## 正确示例

配置的 ignore 列表接受项目的 `v-hover-tooltip` 指令。

```vue annotate="add:2"
<template>
<button type="button" v-hover-tooltip="tip">Copy</button>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/warn-custom-directive": "warn"
    },
    "ruleOptions": {
      "vue/warn-custom-directive": {
        "ignore": [
          "hover-tooltip"
        ]
      }
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [规则选项](/rules/options.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/warn_custom_directive.rs#L44)
