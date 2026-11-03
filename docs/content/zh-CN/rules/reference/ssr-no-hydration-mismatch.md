---
title: "ssr/no-hydration-mismatch"
description: "禁止导致水合不匹配的不确定值。"
---

# `ssr/no-hydration-mismatch`

禁止导致水合不匹配的不确定值。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

模板渲染时执行 `Math.random()`，因此服务端和客户端可能为同一段落生成不同文本。

```vue annotate="remove:2"
<template>
  <p>{{ Math.random() }}</p>
</template>
```

## 正确示例

段落渲染稳定的 `seed` 状态，不再生成新随机结果。在此 Nuxt 风格示例中，`useState` 提供共享状态，初始值是常量 `"stable"`。

```vue annotate="add:1,2,3,4,6"
<script setup lang="ts">
const seed = useState("seed", () => "stable");
</script>

<template>
  <p>{{ seed }}</p>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "ssr/no-hydration-mismatch": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [SSR 规则](../ssr.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_hydration_mismatch.rs#L122)
