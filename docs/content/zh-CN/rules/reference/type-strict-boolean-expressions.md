---
title: "type/strict-boolean-expressions"
description: "要求脚本和模板条件使用安全的布尔表达式。"
---

# `type/strict-boolean-expressions`

要求脚本和模板条件使用安全的布尔表达式。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 脚本和模板中下方所示结构的类型信息。

显式启用 typeAware 和此规则。默认不允许可空数字，允许非空数字。

这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。

## 错误示例

`if (count)` 依赖可空数字绑定的真值性，而非显式布尔检查，也将零与缺失混为一谈。

```vue annotate="remove:3"
<script setup lang="ts">
const count: number | undefined = undefined;
if (count) console.log(count);
</script>
```

## 正确示例

`count !== undefined && count > 0` 分别检查存在性和正值，缩窄可选值后产生显式布尔条件。

```vue annotate="add:3"
<script setup lang="ts">
const count: number | undefined = undefined;
if (count !== undefined && count > 0) console.log(count);
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "type/strict-boolean-expressions": "warn"
      },
      "typeAware": true
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [规则选项](/rules/options.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/strict_boolean_expressions.rs#L7)
