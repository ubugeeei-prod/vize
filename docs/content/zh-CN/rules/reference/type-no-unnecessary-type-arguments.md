---
title: "type/no-unnecessary-type-arguments"
description: "禁止与默认值相同的类型参数。"
---

# `type/no-unnecessary-type-arguments`

禁止与默认值相同的类型参数。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 脚本和模板中下方所示结构的类型信息。

这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。

## 错误示例

显式写出的 number 与类型参数的默认值相同。

```vue annotate="remove:5"
<script setup lang="ts">
function identity<T = number>(value: T): T {
  return value;
}
const fromIdentity = identity<number>(1);
</script>
```

## 正确示例

调用 identity(1) 时会使用类型参数的默认值。

```vue annotate="add:5"
<script setup lang="ts">
function identity<T = number>(value: T): T {
  return value;
}
const fromIdentity = identity(1);
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unnecessary-type-arguments": "warn"
    },
    "typeAware": true
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unnecessary_type_arguments.rs#L5)
