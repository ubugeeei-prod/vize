---
title: "script/require-slots-as-functions"
description: "要求将 $slots 的属性作为函数调用。"
---

# `script/require-slots-as-functions`

要求将 $slots 的属性作为函数调用。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

`render` 返回未调用的 `this.$slots.default`，这是 Vue 2 的 vnode 数组。

```vue annotate="remove:4"
<script lang="ts">
export default {
  render() {
    return this.$slots.default
  }
}
</script>
```

## 正确示例

`render` 调用 `this.$slots.default()` 并使用返回的 vnode。

```vue annotate="add:4"
<script lang="ts">
export default {
  render() {
    return this.$slots.default()
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
      "script/require-slots-as-functions": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_slots_as_functions.rs#L44)
