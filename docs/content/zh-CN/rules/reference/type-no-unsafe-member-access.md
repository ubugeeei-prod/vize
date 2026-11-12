---
title: "type/no-unsafe-member-access"
description: "禁止访问类型为 any 的值的成员。"
---

# `type/no-unsafe-member-access`

禁止访问类型为 any 的值的成员。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 脚本和模板中下方所示结构的类型信息。

这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。

## 错误示例

anyValue 的类型是 any，因此读取 member 是未经检查的属性访问。

```vue annotate="remove:2,3"
<script setup lang="ts">
declare const anyValue: any;
console.log(anyValue.member);
</script>
```

## 正确示例

box 具有已知的 member 属性，因此这次访问会受到检查。

```vue annotate="add:2,3"
<script setup lang="ts">
const box = { member: 1 };
console.log(box.member);
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unsafe-member-access": "warn"
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
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_member_access.rs#L5)
