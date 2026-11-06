---
title: "vue/require-toggle-inside-transition"
description: "要求 `<transition>` 包裹的元素具有切换条件。"
---

# `vue/require-toggle-inside-transition`

要求 `<transition>` 包裹的元素具有切换条件。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

`<Transition>` 内的静态子节点没有条件可见性或动态选择，无法触发进入或离开的变化。

```vue annotate="remove:3"
<template>
<transition>
  <div>content</div>
</transition>
</template>
```

## 正确示例

`v-if="show"` 改变子节点是否存在，为过渡提供进入和离开的边界。

```vue annotate="add:3"
<template>
<transition>
  <div v-if="show">content</div>
</transition>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/require-toggle-inside-transition": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_toggle_inside_transition.rs#L49)
