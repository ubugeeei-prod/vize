---
title: "type/only-throw-error"
description: "要求抛出的值是 Error 一类的值。"
---

# `type/only-throw-error`

要求抛出的值是 Error 一类的值。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 脚本和模板中下方所示结构的类型信息。

这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。

## 错误示例

抛出的值是字符串，不是 Error。

```vue annotate="remove:3"
<script setup lang="ts">
function fail(): never {
  throw "failure";
}
</script>
```

## 正确示例

new Error 为同一段失败文本创建一个 Error。

```vue annotate="add:3"
<script setup lang="ts">
function fail(): never {
  throw new Error("failure");
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
      "type/only-throw-error": "warn"
    },
    "typeAware": true
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/only_throw_error.rs#L5)
