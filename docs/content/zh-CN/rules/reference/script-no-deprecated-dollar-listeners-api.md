---
title: "script/no-deprecated-dollar-listeners-api"
description: "禁止 Vue 3 已移除的 $listeners 实例属性（已合并到 $attrs）。"
---

# `script/no-deprecated-dollar-listeners-api`

禁止 Vue 3 已移除的 $listeners 实例属性（已合并到 $attrs）。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

成员读取和独立参数引用都使用 `$listeners`，Vue 3 在将监听器合并到属性后移除了它。

```vue annotate="remove:4,5,9"
<script>
export default {
  mounted() {
    const vm = this
    return vm.$listeners
  }
}
</script>
<template><Child v-on="$listeners" /></template>
```

## 正确示例

读取改为 `this.$attrs` 和 setup 上下文的 `ctx.attrs`，替代已移除的监听器接口；示例中的接收对象必须存在于周围组件上下文。

```vue annotate="add:4,8"
<script>
export default {
  mounted() {
    return this.$attrs
  }
}
</script>
<template><Child v-bind="$attrs" /></template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-dollar-listeners-api": "error"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_listeners_api.rs#L54)
