---
title: "script/no-expose-after-await"
description: "禁止在 await 之后调用 expose() 和 defineExpose()。"
---

# `script/no-expose-after-await`

禁止在 await 之后调用 expose() 和 defineExpose()。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

`defineExpose()` 在顶层 `await` 之后执行，此时没有当前组件实例，因此不会暴露任何内容。

```vue annotate="remove:2"
<script setup lang="ts">
await loadData()
defineExpose({ reload })
</script>
```

## 正确示例

`defineExpose()` 在第一个 `await` 之前执行，此时组件实例仍是当前实例。

```vue annotate="add:3"
<script setup lang="ts">
defineExpose({ reload })
await loadData()
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "script/no-expose-after-await": "error"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_expose_after_await.rs#L34)
