---
title: "type/no-mixed-enums"
description: "禁止在枚举中混用字符串成员和数值成员。"
---

# `type/no-mixed-enums`

禁止在枚举中混用字符串成员和数值成员。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 脚本和模板中下方所示结构的类型信息。

这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。

## 错误示例

枚举同时包含数值成员和字符串成员。

```vue annotate="remove:4"
<script setup lang="ts">
enum Mixed {
  A = 0,
  B = "b",
}
</script>
```

## 正确示例

两个成员都是数值，因此枚举只有一种值域。

```vue annotate="add:4"
<script setup lang="ts">
enum Mixed {
  A = 0,
  B = 1,
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
      "type/no-mixed-enums": "warn"
    },
    "typeAware": true
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_mixed_enums.rs#L5)
