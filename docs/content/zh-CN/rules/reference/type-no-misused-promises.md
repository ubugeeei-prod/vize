---
title: "type/no-misused-promises"
description: "禁止在并非为处理 Promise 而设计的位置使用 Promise。"
---

# `type/no-misused-promises`

禁止在并非为处理 Promise 而设计的位置使用 Promise。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 脚本和模板中下方所示结构的类型信息。

这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。

## 错误示例

forEach 期望回调返回 void，但 async 回调返回 Promise。

```vue annotate="remove:3"
<script setup lang="ts">
const numbers = [1, 2, 3];
numbers.forEach(async (n) => {
  console.log(n);
});
</script>
```

## 正确示例

回调不是 async，因此它按 forEach 的期望返回 void。

```vue annotate="add:3"
<script setup lang="ts">
const numbers = [1, 2, 3];
numbers.forEach((n) => {
  console.log(n);
});
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-misused-promises": "warn"
    },
    "typeAware": true
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_misused_promises.rs#L5)
