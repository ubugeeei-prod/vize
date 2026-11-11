---
title: "type/no-unnecessary-boolean-literal-compare"
description: "禁止与布尔字面量进行不必要的比较。"
---

# `type/no-unnecessary-boolean-literal-compare`

禁止与布尔字面量进行不必要的比较。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 脚本和模板中下方所示结构的类型信息。

这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。

## 错误示例

flag 已经是布尔值，因此与 true 比较不会增加检查。

```vue annotate="remove:3"
<script setup lang="ts">
function isFlag(flag: boolean): boolean {
  return flag === true;
}
</script>
```

## 正确示例

直接返回 flag 就是在使用这个布尔值。

```vue annotate="add:3"
<script setup lang="ts">
function isFlag(flag: boolean): boolean {
  return flag;
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
      "type/no-unnecessary-boolean-literal-compare": "warn"
    },
    "typeAware": true
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unnecessary_boolean_literal_compare.rs#L5)
