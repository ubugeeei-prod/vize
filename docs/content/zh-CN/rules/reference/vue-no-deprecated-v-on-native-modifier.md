---
title: "vue/no-deprecated-v-on-native-modifier"
description: "禁止在 `v-on` 上使用已弃用的 `.native` 修饰符。"
---

# `vue/no-deprecated-v-on-native-modifier`

禁止在 `v-on` 上使用已弃用的 `.native` 修饰符。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

组件事件处理器使用了已移除的 .native 事件修饰符。

```vue annotate="remove:2,3,4"
<template>
<MyComponent @click.native="handler" />
<MyComponent v-on:click.native="handler" />
<MyComponent @click.native.stop="handler" />
</template>
```

## 正确示例

处理器省略 .native，并保留 .stop 等其他事件修饰符。

```vue annotate="add:2,3"
<template>
<MyComponent @click="handler" />
<MyComponent @click.stop="handler" />
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
        "vue/no-deprecated-v-on-native-modifier": "error"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_on_native_modifier.rs#L45)
