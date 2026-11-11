---
title: "type/await-thenable"
description: "禁止 await 非 thenable 的值。"
---

# `type/await-thenable`

禁止 await 非 thenable 的值。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 脚本和模板中下方所示结构的类型信息。

这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。

## 错误示例

plainNumber 返回数字，因此 await 等待的不是 thenable 值。

```vue annotate="remove:2,3,4,6"
<script setup lang="ts">
function plainNumber(): number {
  return 1;
}
async function run(): Promise<number> {
  return await plainNumber();
}
</script>
```

## 正确示例

Promise.resolve(1) 是 thenable，await 等待的是真正的 Promise。

```vue annotate="add:3"
<script setup lang="ts">
async function run(): Promise<number> {
  return await Promise.resolve(1);
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
      "type/await-thenable": "warn"
    },
    "typeAware": true
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/await_thenable.rs#L5)
