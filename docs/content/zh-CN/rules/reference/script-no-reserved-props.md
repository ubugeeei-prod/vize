---
title: "script/no-reserved-props"
description: "禁止在组件 props 声明中使用保留名称。"
---

# `script/no-reserved-props`

禁止在组件 props 声明中使用保留名称。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

对象形式的 `ref` 和 `$foo`，以及数组形式的 `key`，都是保留 prop 名称。`ref` 和 `key` 是框架控制项，带 `$` 前缀的名称也会被拒绝。

```vue annotate="remove:4,5,8,9,10,11"
<script lang="ts">
export default {
  props: {
    ref: String,   // reserved
    $foo: Number    // `$`-prefixed names are reserved
  }
}

export default {
  props: ['key']    // reserved (array form)
}
</script>
```

## 正确示例

普通 prop 名称 `name` 和 `refValue` 在拼写及前缀上都避开了保留名称。

```vue annotate="add:4,5"
<script lang="ts">
export default {
  props: {
    name: String,
    refValue: Number
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
      "script/no-reserved-props": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_reserved_props.rs#L55)
