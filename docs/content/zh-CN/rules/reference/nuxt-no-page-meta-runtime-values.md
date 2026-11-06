---
title: "nuxt/no-page-meta-runtime-values"
description: "禁止在 `definePageMeta` 的立即求值层使用运行时上下文值；该层在构建时被提取到独立代码块，并在组件 setup 之前运行。"
---

# `nuxt/no-page-meta-runtime-values`

禁止在 `definePageMeta` 的立即求值层使用运行时上下文值；该层在构建时被提取到独立代码块，并在组件 setup 之前运行。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `nuxt` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

构造 `definePageMeta` 对象时立即执行 `useRoute()`，但宏会将元数据提升到 setup 运行时上下文之外。

```vue annotate="remove:2"
<script setup lang="ts">
definePageMeta({ title: useRoute() });
</script>
```

## 正确示例

`validate` 接收回调，因此 `useRoute().params.id` 访问延迟到回调运行时。规则区分延迟执行的函数体和立即求值的元数据值。

```vue annotate="add:2"
<script setup lang="ts">
definePageMeta({ validate: () => Boolean(useRoute().params.id) });
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "nuxt/no-page-meta-runtime-values": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [生态系统与 Nuxt 规则](../ecosystem.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_page_meta_runtime_values.rs#L25)
