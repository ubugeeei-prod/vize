---
title: "type/prefer-reduce-type-parameter"
description: "推荐使用 Array#reduce 的类型参数，而不是对初始值做断言。"
---

# `type/prefer-reduce-type-parameter`

推荐使用 Array#reduce 的类型参数，而不是对初始值做断言。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 脚本和模板中下方所示结构的类型信息。

这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。

## 错误示例

初始值被断言为 number[]，而没有使用 reduce 的类型参数。

```vue annotate="remove:3"
<script setup lang="ts">
const numbers = [1, 2, 3];
const collected = numbers.reduce((all, n) => [...all, n], [] as number[]);
</script>
```

## 正确示例

reduce<number[]> 直接声明累加器类型，不再断言初始数组。

```vue annotate="add:3"
<script setup lang="ts">
const numbers = [1, 2, 3];
const collected = numbers.reduce<number[]>((all, n) => [...all, n], []);
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/prefer-reduce-type-parameter": "warn"
    },
    "typeAware": true
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/prefer_reduce_type_parameter.rs#L5)
