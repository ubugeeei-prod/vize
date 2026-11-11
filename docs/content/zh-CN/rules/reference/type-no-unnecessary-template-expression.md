---
title: "type/no-unnecessary-template-expression"
description: "禁止用不必要的模板字符串只包裹一个表达式。"
---

# `type/no-unnecessary-template-expression`

禁止用不必要的模板字符串只包裹一个表达式。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 脚本和模板中下方所示结构的类型信息。

这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。

## 错误示例

模板里只有 label，因此它并没有拼接出新的字符串。

```vue annotate="remove:3"
<script setup lang="ts">
const label = "a";
const wrapped = `${label}`;
</script>
```

## 正确示例

直接把 label 赋给变量，不需要再用模板包裹。

```vue annotate="add:3"
<script setup lang="ts">
const label = "a";
const wrapped = label;
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unnecessary-template-expression": "warn"
    },
    "typeAware": true
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unnecessary_template_expression.rs#L5)
