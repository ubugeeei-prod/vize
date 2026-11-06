---
title: "vue/valid-v-model"
description: "要求 `v-model` 指令有效。"
---

# `vue/valid-v-model`

要求 `v-model` 指令有效。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

原生 `<div>` 不能像表单控件一样使用 `v-model`，输入框上不带值的指令也没有可写的目标表达式。

```vue annotate="remove:2,3"
<template>
  <div v-model="value"></div>
  <input v-model />
</template>
```

## 正确示例

将 input、select、textarea 或自定义组件绑定到示例中的可写变量。

```vue annotate="add:2,3,4,5"
<template>
  <input v-model="value" />
  <select v-model="selected"></select>
  <textarea v-model="text"></textarea>
  <MyInput v-model="value" />
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/valid-v-model": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_model.rs#L36)
