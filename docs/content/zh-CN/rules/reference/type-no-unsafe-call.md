---
title: "type/no-unsafe-call"
description: "禁止调用类型为 any 的值。"
---

# `type/no-unsafe-call`

禁止调用类型为 any 的值。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 脚本和模板中下方所示结构的类型信息。

这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。

## 错误示例

anyValue 的类型是 any，因此调用它时无法检查被调用者。

```vue annotate="remove:2,3"
<script setup lang="ts">
declare const anyValue: any;
anyValue();
</script>
```

## 正确示例

plainNumber 是函数，因此调用目标是明确的。

```vue annotate="add:2,3,4,5"
<script setup lang="ts">
function plainNumber(): number {
  return 1;
}
plainNumber();
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unsafe-call": "warn"
    },
    "typeAware": true
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_call.rs#L5)
