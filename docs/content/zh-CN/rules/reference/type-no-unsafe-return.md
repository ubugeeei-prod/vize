---
title: "type/no-unsafe-return"
description: "禁止从函数返回 any 类型的值。"
---

# `type/no-unsafe-return`

禁止从函数返回 any 类型的值。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 脚本和模板中下方所示结构的类型信息。

这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。

## 错误示例

函数声明返回 number，但返回的 anyValue 是 any。

```vue annotate="remove:2,4"
<script setup lang="ts">
declare const anyValue: any;
function returnsAny(): number {
  return anyValue;
}
</script>
```

## 正确示例

函数返回数字 1，与注明的返回类型一致。

```vue annotate="add:3"
<script setup lang="ts">
function returnsAny(): number {
  return 1;
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
      "type/no-unsafe-return": "warn"
    },
    "typeAware": true
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_return.rs#L5)
