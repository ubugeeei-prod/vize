---
title: "script/require-prop-types"
description: "要求每个 prop 声明类型。"
---

# `script/require-prop-types`

要求每个 prop 声明类型。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

数组项只声明名称 `status`；`null` 值和空描述对象也没有声明运行时 prop 类型。

```vue annotate="remove:3,4,5,6,8,9"
<script lang="ts">
export default {
  props: ['status']            // array form: no types
}

export default {
  props: {
    status: null,              // no type
    other: {}                  // empty descriptor: no type
  }
}
</script>
```

## 正确示例

`status: String` 提供简写构造器，`other` 在描述对象内提供 `type: Number`，两个 props 现在都有类型声明。

```vue annotate="add:4,5"
<script lang="ts">
export default {
  props: {
    status: String,
    other: { type: Number, default: 0 }
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
      "script/require-prop-types": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_prop_types.rs#L62)
