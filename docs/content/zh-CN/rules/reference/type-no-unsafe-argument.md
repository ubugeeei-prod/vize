---
title: "type/no-unsafe-argument"
description: "禁止用 `any` 类型的值调用函数。"
---

# `type/no-unsafe-argument`

禁止用 `any` 类型的值调用函数。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 脚本和模板中下方所示结构的类型信息。

这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。

## 错误示例

anyValue 的类型是 any，它被传给了期望 number 的参数。

```vue annotate="remove:2,6"
<script setup lang="ts">
declare const anyValue: any;
function takesNumber(value: number): number {
  return value;
}
takesNumber(anyValue);
</script>
```

## 正确示例

调用把数字 1 传给 number 参数。

```vue annotate="add:5"
<script setup lang="ts">
function takesNumber(value: number): number {
  return value;
}
takesNumber(1);
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unsafe-argument": "warn"
    },
    "typeAware": true
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_argument.rs#L5)
