---
title: "type/no-unsafe-unary-minus"
description: "禁止对非 number 且非 bigint 的值使用一元负号。"
---

# `type/no-unsafe-unary-minus`

禁止对非 number 且非 bigint 的值使用一元负号。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 脚本和模板中下方所示结构的类型信息。

这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。

## 错误示例

label 是字符串，因此一元负号并没有作用在 number 或 bigint 上。

```vue annotate="remove:2,3"
<script setup lang="ts">
const label = "a";
const negated = -label;
</script>
```

## 正确示例

count 是数字，因此一元负号否定的是这个数字。

```vue annotate="add:2,3"
<script setup lang="ts">
const count = 1;
const negated = -count;
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unsafe-unary-minus": "warn"
    },
    "typeAware": true
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_unary_minus.rs#L5)
