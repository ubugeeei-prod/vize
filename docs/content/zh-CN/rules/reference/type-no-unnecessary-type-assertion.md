---
title: "type/no-unnecessary-type-assertion"
description: "禁止不改变表达式类型的类型断言。"
---

# `type/no-unnecessary-type-assertion`

禁止不改变表达式类型的类型断言。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 脚本和模板中下方所示结构的类型信息。

这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。

## 错误示例

known 已经是 number，因此再断言为 number 不会改变它的类型。

```vue annotate="remove:3"
<script setup lang="ts">
const known: number = 1;
const asserted = known as number;
</script>
```

## 正确示例

直接使用 known，保留已注明的 number 类型。

```vue annotate="add:3"
<script setup lang="ts">
const known: number = 1;
const asserted = known;
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unnecessary-type-assertion": "warn"
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
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unnecessary_type_assertion.rs#L5)
