---
title: "type/no-unnecessary-type-parameters"
description: "禁止在签名中只使用一次的类型参数。"
---

# `type/no-unnecessary-type-parameters`

禁止在签名中只使用一次的类型参数。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 脚本和模板中下方所示结构的类型信息。

这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。

## 错误示例

T 在 logValue 的签名中只出现一次，因此没有关联任何类型。

```vue annotate="remove:2"
<script setup lang="ts">
function logValue<T>(value: T): void {
  console.info(value);
}
logValue(1);
</script>
```

## 正确示例

把 value 声明为 unknown，无需类型参数即可接受相同的参数。

```vue annotate="add:2"
<script setup lang="ts">
function logValue(value: unknown): void {
  console.info(value);
}
logValue(1);
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unnecessary-type-parameters": "warn"
    },
    "typeAware": true
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unnecessary_type_parameters.rs#L5)
