---
title: "type/return-await"
description: "要求根据上下文和配置一致地使用 `return await`。"
---

# `type/return-await`

要求根据上下文和配置一致地使用 `return await`。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 脚本和模板中下方所示结构的类型信息。

这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。

## 错误示例

在 try 中不 await 就返回 getNumber()，这个 catch 接收不到拒绝原因。

```vue annotate="remove:7"
<script setup lang="ts">
function getNumber(): Promise<number> {
  return Promise.resolve(1);
}
async function returnsInTry(): Promise<number> {
  try {
    return getNumber();
  } catch {
    return 0;
  }
}
</script>
```

## 正确示例

return await getNumber() 会把拒绝留在同一个 try 和 catch 中。

```vue annotate="add:7"
<script setup lang="ts">
function getNumber(): Promise<number> {
  return Promise.resolve(1);
}
async function returnsInTry(): Promise<number> {
  try {
    return await getNumber();
  } catch {
    return 0;
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
      "type/return-await": "warn"
    },
    "typeAware": true
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/return_await.rs#L5)
