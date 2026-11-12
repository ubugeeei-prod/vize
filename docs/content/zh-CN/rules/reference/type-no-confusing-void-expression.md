---
title: "type/no-confusing-void-expression"
description: "禁止在容易混淆的值位置使用 void 类型的表达式。"
---

# `type/no-confusing-void-expression`

禁止在容易混淆的值位置使用 void 类型的表达式。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 脚本和模板中下方所示结构的类型信息。

这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。

## 错误示例

logVoid() 返回 void，但初始化语句把这次调用当作一个值。

```vue annotate="remove:3"
<script setup lang="ts">
function logVoid(): void {}
const value = logVoid();
</script>
```

## 正确示例

把 logVoid() 写成语句后，void 结果只用于其副作用。

```vue annotate="add:3"
<script setup lang="ts">
function logVoid(): void {}
logVoid();
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-confusing-void-expression": "warn"
    },
    "typeAware": true
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [规则选项](/rules/options.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_confusing_void_expression.rs#L5)
