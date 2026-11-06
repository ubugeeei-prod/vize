---
title: "script/no-deprecated-data-object-declaration"
description: "禁止将对象字面量作为组件 data 选项（Vue 3 要求函数）。"
---

# `script/no-deprecated-data-object-declaration`

禁止将对象字面量作为组件 data 选项（Vue 3 要求函数）。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

Options API 的 `data` 选项是对象字面量，这是 Vue 3 不再接受的 Vue 2 形式。

```vue annotate="remove:3,4,5"
<script lang="ts">
export default {
  // `data` must be a function in Vue 3, not an object literal.
  data: {
    count: 0
  }
}
</script>
```

## 正确示例

`data()` 返回新的 `{ count: 0 }` 对象，提供 Vue 3 所需的函数式 data 声明。

```vue annotate="add:3,4"
<script lang="ts">
export default {
  data() {
    return { count: 0 }
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
      "script/no-deprecated-data-object-declaration": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_data_object_declaration.rs#L49)
