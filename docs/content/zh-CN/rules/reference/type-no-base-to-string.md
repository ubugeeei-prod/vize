---
title: "type/no-base-to-string"
description: "禁止把会回退到 Object.prototype.toString() 的值转换成字符串。"
---

# `type/no-base-to-string`

禁止把会回退到 Object.prototype.toString() 的值转换成字符串。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 脚本和模板中下方所示结构的类型信息。

这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。

## 错误示例

plain 是普通对象，因此 toString() 会回退到 Object.prototype.toString()。

```vue annotate="remove:2,3"
<script setup lang="ts">
const plain = { a: 1 };
plain.toString();
</script>
```

## 正确示例

label 已经是字符串，因此 toString() 使用字符串自身的转换。

```vue annotate="add:2,3"
<script setup lang="ts">
const label = "a";
label.toString();
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-base-to-string": "warn"
    },
    "typeAware": true
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_base_to_string.rs#L5)
