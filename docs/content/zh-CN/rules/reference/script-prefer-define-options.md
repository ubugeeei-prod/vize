---
title: "script/prefer-define-options"
description: "优先使用 defineOptions()，而不是仅设置 name/inheritAttrs 的普通 &lt;script&gt;。"
---

# `script/prefer-define-options`

优先使用 defineOptions()，而不是仅设置 name/inheritAttrs 的普通 &lt;script&gt;。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

普通脚本唯一有实质作用的语句导出只含 `name` 和 `inheritAttrs` 的对象；这些选项可通过 `defineOptions` 表达。

```vue annotate="remove:2"
<script lang="ts">
export default { name: 'MyComponent', inheritAttrs: false }
</script>
```

## 正确示例

示例中的 `data()` 方法让脚本包含实际 Options API 逻辑，因此不在此规则保守的仅选项建议范围内。此正确示例展示允许的例外；直接迁移则应在 `<script setup>` 中使用 `defineOptions({ name: 'MyComponent', inheritAttrs: false })`。

```vue annotate="add:2,3,4,5,6"
<script lang="ts">
// Real options logic — keep the plain script.
export default {
  name: 'MyComponent',
  data() { return { count: 0 } },
}
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
        "script/prefer-define-options": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_define_options.rs#L52)
