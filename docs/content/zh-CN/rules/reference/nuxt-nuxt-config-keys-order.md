---
title: "nuxt/nuxt-config-keys-order"
description: "优先使用推荐的 Nuxt 配置属性顺序。"
---

# `nuxt/nuxt-config-keys-order`

优先使用推荐的 Nuxt 配置属性顺序。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `nuxt` | 支持 | 无 |

适用范围：Nuxt 配置文件（nuxt.config.ts）。

## 错误示例

配置将 `ssr` 放在 `modules` 之前，与规则推荐的 Nuxt 配置键顺序相反。

`nuxt.config.ts`

```ts annotate="remove:1"
export default defineNuxtConfig({ ssr: true, modules: [] });
```

## 正确示例

将 `modules` 放在 `ssr` 之前，在保留两个值的同时满足规定顺序；修复改变布局，不改变选项含义。

`nuxt.config.ts`

```ts annotate="add:1"
export default defineNuxtConfig({ modules: [], ssr: true });
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "nuxt/nuxt-config-keys-order": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [生态系统与 Nuxt 规则](../ecosystem.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/nuxt_config_keys_order.rs#L24)
