---
title: "script/no-deprecated-dollar-scopedslots-api"
description: "禁止 Vue 3 已移除的 $scopedSlots 实例属性（使用 $slots）。"
---

# `script/no-deprecated-dollar-scopedslots-api`

禁止 Vue 3 已移除的 $scopedSlots 实例属性（使用 $slots）。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

`this.$scopedSlots`、`ctx.$scopedSlots` 和独立的 `$scopedSlots` 引用使用了 Vue 3 已移除的 Vue 2 作用域插槽 API。

```vue annotate="remove:2,3,4"
<script setup lang="ts">
const header = this.$scopedSlots.header
const footer = ctx.$scopedSlots.footer
render($scopedSlots.default)
</script>
```

## 正确示例

将 `$scopedSlots` 替换为 `$slots`，使用统一插槽接口。示例移除弃用写法，并未为接收对象建立 setup 上下文。

```vue annotate="add:2,3,4"
<script setup lang="ts">
const header = this.$slots.header
const footer = ctx.$slots.footer
render($slots.default)
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-deprecated-dollar-scopedslots-api": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_scopedslots_api.rs#L45)
