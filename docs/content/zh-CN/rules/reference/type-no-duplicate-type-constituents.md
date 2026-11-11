---
title: "type/no-duplicate-type-constituents"
description: "禁止联合类型或交叉类型中的重复成员。"
---

# `type/no-duplicate-type-constituents`

禁止联合类型或交叉类型中的重复成员。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 脚本和模板中下方所示结构的类型信息。

这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。

## 错误示例

string 在同一个联合类型中出现两次，第二个成员没有增加任何信息。

```vue annotate="remove:2"
<script setup lang="ts">
type Dup = string | string;
</script>
```

## 正确示例

类型别名只保留一个 string 成员。

```vue annotate="add:2"
<script setup lang="ts">
type Dup = string;
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-duplicate-type-constituents": "warn"
    },
    "typeAware": true
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_duplicate_type_constituents.rs#L5)
