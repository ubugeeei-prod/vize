---
title: "type/require-await"
description: "禁止不包含 await 表达式的 async 函数。"
---

# `type/require-await`

禁止不包含 await 表达式的 async 函数。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 脚本和模板中下方所示结构的类型信息。

这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。

## 错误示例

noAwait 是 async，但它的函数体从未 await 任何值。

```vue annotate="remove:2,3"
<script setup lang="ts">
async function noAwait(): Promise<number> {
  return 1;
}
</script>
```

## 正确示例

withAwait 在返回之前 await 了 Promise.resolve(1)。

```vue annotate="add:2,3"
<script setup lang="ts">
async function withAwait(): Promise<number> {
  return await Promise.resolve(1);
}
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/require-await": "warn"
    },
    "typeAware": true
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/require_await.rs#L5)
