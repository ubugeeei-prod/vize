---
title: "script/valid-next-tick"
description: "要求 nextTick() 调用结果被等待、链式处理，或提供回调。"
---

# `script/valid-next-tick`

要求 nextTick() 调用结果被等待、链式处理，或提供回调。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

导入的 `nextTick()` 是无回调的独立表达式，其返回 Promise 被忽略，没有操作等待 DOM 刷新。

```vue annotate="remove:3"
<script setup lang="ts">
import { nextTick } from "vue";
nextTick();
</script>
```

## 正确示例

`await nextTick()` 使用该 Promise，显式等待下一次 DOM 更新，再继续执行后续 setup 代码。

```vue annotate="add:3"
<script setup lang="ts">
import { nextTick } from "vue";
await nextTick();
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/valid-next-tick": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_next_tick.rs#L54)
