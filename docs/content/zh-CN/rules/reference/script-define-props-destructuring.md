---
title: "script/define-props-destructuring"
description: "要求 &lt;script setup&gt; 中的 defineProps 解构风格保持一致。"
---

# `script/define-props-destructuring`

要求 &lt;script setup&gt; 中的 defineProps 解构风格保持一致。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

`defineProps` 被赋给单一的 `props` 绑定，而非解构，不符合默认的解构偏好。

```vue annotate="remove:2"
<script setup lang="ts">
const props = defineProps<{ foo: string }>()
</script>
```

## 正确示例

对象模式直接绑定 `foo` 和 `bar`，并为可选的 `bar` 提供默认值。这依赖 Vue 3.5+ 的响应式 props 解构；可配置的 `never` 模式偏好相反形式。

```vue annotate="add:2"
<script setup lang="ts">
const { foo, bar = 'default' } = defineProps<{ foo: string; bar?: string }>()
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
        "script/define-props-destructuring": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [规则选项](/rules/options.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_props_destructuring.rs#L29)
