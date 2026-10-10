---
title: "SSR 规则"
---

# SSR 规则

本页集中列出所有相关规则的用途、配置、错误示例和正确示例，无需跳转页面查看对照。高亮行标示修改；复制代码时保留完整源代码。每条规则注明适用范围和当前支持限制；规范示例并不保证当前实现会产生诊断。

<span id="ssr规则"></span>

| 规则 | 示例 | 用途 |
| --- | --- | --- |
| [`ssr/no-browser-globals-in-ssr`](#ssr-no-browser-globals-in-ssr) | [错误示例](#ssr-no-browser-globals-in-ssr-bad) · [正确示例](#ssr-no-browser-globals-in-ssr-good) | 禁止 SSR 上下文中的浏览器专用全局变量 |
| [`ssr/no-hydration-mismatch`](#ssr-no-hydration-mismatch) | [错误示例](#ssr-no-hydration-mismatch-bad) · [正确示例](#ssr-no-hydration-mismatch-good) | 禁止导致水合不匹配的不确定值 |

[全部规则](./all.md) · [规则选项](/rules/options.md) · [ESLint 迁移对应表](/rules/migration.md) · [项目检查](./cross-file.md) · [组件间属性传递](/rules/project/vue-cross-file-attrs-fallthrough.md)

### `ssr/no-browser-globals-in-ssr`

禁止 SSR 上下文中的浏览器专用全局变量

[错误示例](#ssr-no-browser-globals-in-ssr-bad) · [正确示例](#ssr-no-browser-globals-in-ssr-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ssr/no-browser-globals-in-ssr": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ssr-no-browser-globals-in-ssr-bad"></span>

**错误示例**

setup 立即读取 `window.innerWidth`，但组件在服务器运行时不存在 `window`。

```vue annotate="remove:2"
<script setup lang="ts">
const width = window.innerWidth;
</script>
```

<span id="ssr-no-browser-globals-in-ssr-good"></span>

**正确示例**

初始宽度是服务端安全的 ref 值，浏览器访问移到 `onMounted`，在客户端运行而不在 SSR setup 期间运行。

```vue annotate="add:2,3,4,5,6"
<script setup lang="ts">
const width = ref(0);

onMounted(() => {
  width.value = window.innerWidth;
});
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_browser_globals_in_ssr.rs#L158) · [全部规则](all.md)

### `ssr/no-hydration-mismatch`

禁止导致水合不匹配的不确定值

[错误示例](#ssr-no-hydration-mismatch-bad) · [正确示例](#ssr-no-hydration-mismatch-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ssr/no-hydration-mismatch": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ssr-no-hydration-mismatch-bad"></span>

**错误示例**

模板渲染时执行 `Math.random()`，因此服务端和客户端可能为同一段落生成不同文本。

```vue annotate="remove:2"
<template>
  <p>{{ Math.random() }}</p>
</template>
```

<span id="ssr-no-hydration-mismatch-good"></span>

**正确示例**

段落渲染稳定的 `seed` 状态，不再生成新随机结果。在此 Nuxt 风格示例中，`useState` 提供共享状态，初始值是常量 `"stable"`。

```vue annotate="add:1,2,3,4,6"
<script setup lang="ts">
const seed = useState("seed", () => "stable");
</script>

<template>
  <p>{{ seed }}</p>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_hydration_mismatch.rs#L122) · [全部规则](all.md)
