---
title: "ecosystem/void-link-valid-method"
description: "验证 Void Vue Link 的静态 method prop。"
---

# `ecosystem/void-link-valid-method`

验证 Void Vue Link 的静态 method prop。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `ecosystem` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

DELETE 操作请求了预取，但预取适用于导航请求。

```vue annotate="remove:6"
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/posts/1" method="DELETE" prefetch>Delete</Link>
</template>
```

## 正确示例

移除 prefetch，保留 DELETE 操作，不再预取该非 GET 请求。

```vue annotate="add:6"
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/posts/1" method="DELETE">Delete</Link>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "ecosystem/void-link-valid-method": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [生态系统与 Nuxt 规则](../ecosystem.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/void_link_valid_method.rs#L14)
