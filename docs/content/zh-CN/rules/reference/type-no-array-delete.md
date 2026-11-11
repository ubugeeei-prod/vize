---
title: "type/no-array-delete"
description: "禁止对类数组值使用 delete 删除元素。"
---

# `type/no-array-delete`

禁止对类数组值使用 delete 删除元素。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 脚本和模板中下方所示结构的类型信息。

这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。

## 错误示例

delete numbers[0] 删除数组元素并留下空洞，而不会重新排列其余元素。

```vue annotate="remove:3"
<script setup lang="ts">
const numbers = [1, 2, 3];
delete numbers[0];
</script>
```

## 正确示例

splice(0, 1) 删除第一个元素，并保持数组连续。

```vue annotate="add:3"
<script setup lang="ts">
const numbers = [1, 2, 3];
numbers.splice(0, 1);
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-array-delete": "warn"
    },
    "typeAware": true
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_array_delete.rs#L5)
