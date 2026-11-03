---
title: "script/no-required-prop-with-default"
description: "禁止 prop 同时具有 required: true 和默认值。"
---

# `script/no-required-prop-with-default`

禁止 prop 同时具有 required: true 和默认值。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

`title` 既是必需项，又具有后备值 `"Untitled"`，混合了必需输入约定和面向缺失输入的默认值。

```vue annotate="remove:2"
<script lang="ts">
export default { props: { title: { type: String, required: true, default: "Untitled" } } };
</script>
```

## 正确示例

移除 `required: true`，使 `title` 可选，并保留 `"Untitled"` 作为一致的后备值。

```vue annotate="add:2"
<script lang="ts">
export default { props: { title: { type: String, default: "Untitled" } } };
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
        "script/no-required-prop-with-default": "error"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_required_prop_with_default.rs#L34)
