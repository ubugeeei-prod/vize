---
title: "type/no-implied-eval"
description: "禁止基于字符串的动态代码执行 API。"
---

# `type/no-implied-eval`

禁止基于字符串的动态代码执行 API。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 脚本和模板中下方所示结构的类型信息。

这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。

## 错误示例

setTimeout 接收字符串，这段字符串会作为动态代码执行。

```vue annotate="remove:2,3"
<script setup lang="ts">
function logVoid(): void {}
setTimeout("logVoid()", 10);
</script>
```

## 正确示例

setTimeout 接收函数，因此不会求值字符串。

```vue annotate="add:2"
<script setup lang="ts">
setTimeout(() => {}, 10);
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-implied-eval": "warn"
    },
    "typeAware": true
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_implied_eval.rs#L5)
