---
title: "type/no-generated-empty-object-type"
description: "禁止由类型运算生成的空对象类型。"
---

# `type/no-generated-empty-object-type`

禁止由类型运算生成的空对象类型。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 脚本和模板中下方所示结构的类型信息。

这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。

## 错误示例

Omit 去掉了唯一的键，因此别名会变成空对象类型。

```vue annotate="remove:2"
<script setup lang="ts">
type Emptied = Omit<{ a: number }, "a">;
</script>
```

## 正确示例

Omit 去掉 a 后仍保留 b，因此别名不是空对象类型。

```vue annotate="add:2"
<script setup lang="ts">
type Kept = Omit<{ a: number; b: string }, "a">;
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-generated-empty-object-type": "warn"
    },
    "typeAware": true
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_generated_empty_object_type.rs#L5)
