---
title: "type/use-unknown-in-catch-callback-variable"
description: "要求 Promise 的 catch 回调参数使用显式的 unknown 注解。"
---

# `type/use-unknown-in-catch-callback-variable`

要求 Promise 的 catch 回调参数使用显式的 unknown 注解。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 脚本和模板中下方所示结构的类型信息。

这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。

## 错误示例

catch 回调给出了 error，却没有显式的 unknown 注解。

```vue annotate="remove:5"
<script setup lang="ts">
function getNumber(): Promise<number> {
  return Promise.resolve(1);
}
getNumber().catch((error) => {
  console.log(error);
});
</script>
```

## 正确示例

catch 回调把 error 注明为 unknown。

```vue annotate="add:5"
<script setup lang="ts">
function getNumber(): Promise<number> {
  return Promise.resolve(1);
}
getNumber().catch((error: unknown) => {
  console.log(error);
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
      "type/use-unknown-in-catch-callback-variable": "warn"
    },
    "typeAware": true
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/use_unknown_in_catch_callback_variable.rs#L5)
