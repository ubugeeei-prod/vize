---
title: "type/no-unsafe-enum-comparison"
description: "禁止把枚举值与另一枚举域的值比较。"
---

# `type/no-unsafe-enum-comparison`

禁止把枚举值与另一枚举域的值比较。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 脚本和模板中下方所示结构的类型信息。

这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。

## 错误示例

value 是数字，因此把它和枚举成员 Color.Red 比较跨越了值域。

```vue annotate="remove:6"
<script setup lang="ts">
enum Color {
  Red = 0,
  Green = 1,
}
function isRed(value: number): boolean {
  return value === Color.Red;
}
</script>
```

## 正确示例

value 是 Color，因此比较停留在 Color 枚举之内。

```vue annotate="add:6"
<script setup lang="ts">
enum Color {
  Red = 0,
  Green = 1,
}
function isRed(value: Color): boolean {
  return value === Color.Red;
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
      "type/no-unsafe-enum-comparison": "warn"
    },
    "typeAware": true
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_enum_comparison.rs#L5)
