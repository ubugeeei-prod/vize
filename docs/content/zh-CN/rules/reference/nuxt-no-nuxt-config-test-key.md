---
title: "nuxt/no-nuxt-config-test-key"
description: "禁止在 Nuxt 配置中设置 `test` 键。"
---

# `nuxt/no-nuxt-config-test-key`

禁止在 Nuxt 配置中设置 `test` 键。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `nuxt` | 不支持 | 无 |

适用范围：Nuxt 配置文件（nuxt.config.ts）。

## 错误示例

导出的 Nuxt 配置将标识符键 `test` 设为布尔值 `true`，这是此规则拒绝的旧配置形式。

`nuxt.config.ts`

```ts annotate="remove:1"
export default defineNuxtConfig({ test: true });
```

## 正确示例

空配置移除了该布尔 `test` 属性。此示例并不禁止测试配置对象。

`nuxt.config.ts`

```ts annotate="add:1"
export default defineNuxtConfig({});
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "nuxt/no-nuxt-config-test-key": "error"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [生态系统与 Nuxt 规则](../ecosystem.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_nuxt_config_test_key.rs#L16)
