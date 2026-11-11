---
title: "type/restrict-plus-operands"
description: "要求加号的操作数明确兼容。"
---

# `type/restrict-plus-operands`

要求加号的操作数明确兼容。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 脚本和模板中下方所示结构的类型信息。

这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。

## 错误示例

label 是字符串，numbers 是数组，因此加号两边不是兼容的原始值。

```vue annotate="remove:3,4"
<script setup lang="ts">
const label = "a";
const numbers = [1, 2, 3];
const mixedSum = label + numbers;
</script>
```

## 正确示例

label 和 extra 都是字符串，因此加号两边的类型一致。

```vue annotate="add:3,4"
<script setup lang="ts">
const label = "a";
const extra = "b";
const mixedSum = label + extra;
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/restrict-plus-operands": "warn"
    },
    "typeAware": true
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/restrict_plus_operands.rs#L5)
