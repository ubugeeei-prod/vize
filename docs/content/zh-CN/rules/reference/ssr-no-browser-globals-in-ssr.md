---
title: "ssr/no-browser-globals-in-ssr"
description: "禁止 SSR 上下文中的浏览器专用全局变量。"
---

# `ssr/no-browser-globals-in-ssr`

禁止 SSR 上下文中的浏览器专用全局变量。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

setup 立即读取 `window.innerWidth`，但组件在服务器运行时不存在 `window`。

```vue annotate="remove:2"
<script setup lang="ts">
const width = window.innerWidth;
</script>
```

## 正确示例

初始宽度是服务端安全的 ref 值，浏览器访问移到 `onMounted`，在客户端运行而不在 SSR setup 期间运行。

```vue annotate="add:2,3,4,5,6"
<script setup lang="ts">
const width = ref(0);

onMounted(() => {
  width.value = window.innerWidth;
});
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "ssr/no-browser-globals-in-ssr": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [SSR 规则](../ssr.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_browser_globals_in_ssr.rs#L157)
