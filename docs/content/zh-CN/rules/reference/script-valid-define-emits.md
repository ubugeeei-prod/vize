---
title: "script/valid-define-emits"
description: "要求 defineEmits() 用法有效（不同时提供类型和运行时参数、不引用局部变量、只调用一次）。"
---

# `script/valid-define-emits`

要求 defineEmits() 用法有效（不同时提供类型和运行时参数、不引用局部变量、只调用一次）。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

同一 `defineEmits` 调用同时提供类型参数和运行时数组 `["save"]`，混合两种互斥声明。

```vue annotate="remove:2"
<script setup lang="ts">
defineEmits<{ save: [] }>(["save"]);
</script>
```

## 正确示例

移除运行时参数，只保留一个针对 `save` 的类型事件声明。

```vue annotate="add:2"
<script setup lang="ts">
defineEmits<{ save: [] }>();
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/valid-define-emits": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_emits.rs#L47)
