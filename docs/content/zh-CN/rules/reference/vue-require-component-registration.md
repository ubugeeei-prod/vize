---
title: "vue/require-component-registration"
description: "要求显式导入或注册组件。"
---

# `vue/require-component-registration`

要求显式导入或注册组件。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `opinionated` | 不支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

列出由应用插件或 Musea previewSetup 提供的明确组件名称。接受 PascalCase 和 kebab-case 写法；不解析正则表达式。设置选项不会启用规则。后面的配置层会替换列表；空列表会清除继承的名称。

## 错误示例

`MissingWidget` 既未注册，也不在配置的全局组件允许列表中。

```vue annotate="remove:2"
<template>
<MissingWidget />
</template>
```

## 正确示例

`MyButton` 列在示例的 `globals` 选项中。该选项让已知的全局组件免于此检查，并不会注册或导入组件。

```vue annotate="add:2"
<template>
<MyButton />
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/require-component-registration": "warn"
    },
    "ruleOptions": {
      "vue/require-component-registration": {
        "globals": [
          "MyButton",
          "MyIcon"
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
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/require_component_registration.rs#L59)
