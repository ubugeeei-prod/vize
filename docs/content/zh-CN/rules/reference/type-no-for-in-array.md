---
title: "type/no-for-in-array"
description: "禁止对类数组值使用 for-in 遍历。"
---

# `type/no-for-in-array`

禁止对类数组值使用 for-in 遍历。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 脚本和模板中下方所示结构的类型信息。

这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。

## 错误示例

for-in 遍历的是数组的键，而不是元素值。

```vue annotate="remove:4,5"
<script setup lang="ts">
const numbers = [1, 2, 3];
function loopsKeys(): void {
  for (const key in numbers) {
    console.log(key);
  }
}
</script>
```

## 正确示例

for-of 遍历数组中的数值元素。

```vue annotate="add:4,5"
<script setup lang="ts">
const numbers = [1, 2, 3];
function loopsKeys(): void {
  for (const n of numbers) {
    console.log(n);
  }
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
      "type/no-for-in-array": "warn"
    },
    "typeAware": true
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_for_in_array.rs#L5)
