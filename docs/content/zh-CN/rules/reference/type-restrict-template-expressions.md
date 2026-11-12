---
title: "type/restrict-template-expressions"
description: "要求模板字符串中的表达式与字符串兼容。"
---

# `type/restrict-template-expressions`

要求模板字符串中的表达式与字符串兼容。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 脚本和模板中下方所示结构的类型信息。

这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。

## 错误示例

模板插入了数组，这不是与字符串兼容的表达式。

```vue annotate="remove:2,3"
<script setup lang="ts">
const numbers = [1, 2, 3];
const listText = `${numbers}`;
</script>
```

## 正确示例

模板插入的 label 是字符串。

```vue annotate="add:2,3"
<script setup lang="ts">
const label = "a";
const listText = `${label}`;
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/restrict-template-expressions": "warn"
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
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/restrict_template_expressions.rs#L5)
