---
title: "script/no-with-defaults"
description: "不建议使用 withDefaults，优先使用解构默认值（Vue 3.5+）。"
---

# `script/no-with-defaults`

不建议使用 withDefaults，优先使用解构默认值（Vue 3.5+）。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

`withDefaults` 包装带类型的 props 声明，仅为 `count` 和 `name` 提供默认值，没有使用此处偏好的 Vue 3.5+ 解构默认值风格。

```vue annotate="remove:2"
<script setup lang="ts">
const props = withDefaults(defineProps<{ count?: number; name?: string }>(), { count: 0, name: "Ada" });
</script>
```

## 正确示例

解构模式将 `count = 0` 和 `name = "Ada"` 放在对应绑定旁，并移除 `withDefaults` 包装。

```vue annotate="add:2"
<script setup lang="ts">
const { count = 0, name = "Ada" } = defineProps<{ count?: number; name?: string }>();
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-with-defaults": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_with_defaults.rs#L41)
