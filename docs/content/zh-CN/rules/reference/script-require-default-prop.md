---
title: "script/require-default-prop"
description: "要求每个可选的非 Boolean prop 具有默认值。"
---

# `script/require-default-prop`

要求每个可选的非 Boolean prop 具有默认值。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `opinionated`, `nuxt` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

`name` 和 `age` 是没有默认值的可选非 Boolean 运行时 props，省略输入时的值未指定。

```vue annotate="remove:4,5,6"
<script lang="ts">
export default {
  props: {
    // optional, non-Boolean, no default
    name: String,
    age: { type: Number },
  }
}
</script>
```

## 正确示例

`name` 获得 `default: ''`。`enabled` 使用 Boolean 隐式的 false 默认值，必需的 `id` 无需后备值，展示了两种豁免。

```vue annotate="add:4,5,6"
<script lang="ts">
export default {
  props: {
    name: { type: String, default: '' },
    enabled: Boolean,                 // Boolean defaults to false
    id: { type: Number, required: true },
  }
}
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/require-default-prop": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_default_prop.rs#L62)
