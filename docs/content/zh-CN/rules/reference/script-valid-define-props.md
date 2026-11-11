---
title: "script/valid-define-props"
description: "要求 defineProps() 用法有效（只调用一次、不同时提供类型和运行时参数、不引用局部变量）。"
---

# `script/valid-define-props`

要求 defineProps() 用法有效（只调用一次、不同时提供类型和运行时参数、不引用局部变量）。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

同一 `defineProps` 调用同时提供类型参数 `{ title: string }` 和运行时参数 `{ title: String }`，编译器不允许二者组合。

```vue annotate="remove:2"
<script setup lang="ts">
defineProps<{ title: string }>({ title: String });
</script>
```

## 正确示例

移除运行时对象，只保留一个针对 `title` 的类型声明，不再组合两种声明形式。

```vue annotate="add:2"
<script setup lang="ts">
defineProps<{ title: string }>();
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/valid-define-props": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_props.rs#L46)
