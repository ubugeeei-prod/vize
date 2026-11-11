---
title: "type/no-deprecated"
description: "禁止使用已弃用的声明。"
---

# `type/no-deprecated`

禁止使用已弃用的声明。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 脚本和模板中下方所示结构的类型信息。

这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。

## 错误示例

legacy 标有 @deprecated，调用仍然使用这个声明。

```vue annotate="remove:2,3,6"
<script setup lang="ts">
/** @deprecated use current() */
function legacy(): number {
  return 1;
}
const legacyValue = legacy();
</script>
```

## 正确示例

current 没有弃用标记，因此调用使用的是仍受支持的声明。

```vue annotate="add:2,5"
<script setup lang="ts">
function current(): number {
  return 1;
}
const currentValue = current();
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-deprecated": "warn"
    },
    "typeAware": true
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_deprecated.rs#L5)
