---
title: "petite-vue/no-unsupported-directive"
description: "禁止 petite-vue 不支持的指令。"
---

# `petite-vue/no-unsupported-directive`

禁止 petite-vue 不支持的指令。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | 无 | 不支持 | 无 |

适用范围：检测为 petite-vue 的 HTML 文档；普通 Vue SFC 不在此规则范围内。

## 错误示例

`v-memo`、`v-slot:header` 和自定义 `v-my-directive` 不在 petite-vue 支持的指令列表中。petite-vue 脚本将此 HTML 标识为相应方言。

```html annotate="remove:3,4,5"
<!doctype html>
<html><body>
<div v-memo="[a, b]"></div>
<template v-slot:header></template>
<div v-my-directive></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

## 正确示例

替换后使用受支持的 `v-scope`、`v-effect`、`v-if`、`v-bind` 和 `v-on` 语法，不再依赖不支持的指令。

```html annotate="add:3,4"
<!doctype html>
<html><body>
<div v-scope="{ count: 0 }" v-effect="console.log(count)"></div>
<div v-if="ok" v-bind:title="title" @click="count++"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "petite-vue/no-unsupported-directive": "error"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [petite-vue 规则](../petite-vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/no_unsupported_directive.rs#L43)
