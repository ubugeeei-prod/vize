---
title: "nuxt/prefer-import-meta"
description: "优先使用 `import.meta.*`，而不是 `process.*`。"
---

# `nuxt/prefer-import-meta`

优先使用 `import.meta.*`，而不是 `process.*`。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `nuxt` | 支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

`process.client` 使用旧 Nuxt 环境标志，规则要求将其迁移到 `import.meta`。

```vue annotate="remove:2"
<script setup lang="ts">
if (process.client) console.log("browser");
</script>
```

## 正确示例

`import.meta.client` 通过替代环境标志显式保留仅浏览器执行的分支。

```vue annotate="add:2"
<script setup lang="ts">
if (import.meta.client) console.log("browser");
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
        "nuxt/prefer-import-meta": "error"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [生态系统与 Nuxt 规则](../ecosystem.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_meta.rs#L18)
