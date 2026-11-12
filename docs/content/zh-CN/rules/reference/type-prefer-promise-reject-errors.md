---
title: "type/prefer-promise-reject-errors"
description: "要求 Promise 的拒绝原因是 Error 一类的值。"
---

# `type/prefer-promise-reject-errors`

要求 Promise 的拒绝原因是 Error 一类的值。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 脚本和模板中下方所示结构的类型信息。

这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。

## 错误示例

Promise.reject 接收字符串，这不是 Error 原因。

```vue annotate="remove:2"
<script setup lang="ts">
const rejected = Promise.reject("failure");
</script>
```

## 正确示例

Promise.reject 接收的是同一段失败文本对应的 Error。

```vue annotate="add:2"
<script setup lang="ts">
const rejected = Promise.reject(new Error("failure"));
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/prefer-promise-reject-errors": "warn"
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
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/prefer_promise_reject_errors.rs#L5)
