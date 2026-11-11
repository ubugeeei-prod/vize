---
title: "type/prefer-return-this-type"
description: "要求在只返回 `this` 类型时使用 `this`。"
---

# `type/prefer-return-this-type`

要求在只返回 `this` 类型时使用 `this`。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 脚本和模板中下方所示结构的类型信息。

这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。

## 错误示例

setValue 返回 this，但声明的返回类型是类名 Builder。

```vue annotate="remove:3"
<script setup lang="ts">
class Builder {
  setValue(): Builder {
    return this;
  }
}
</script>
```

## 正确示例

返回类型 this 与方法实际返回的值一致。

```vue annotate="add:3"
<script setup lang="ts">
class Builder {
  setValue(): this {
    return this;
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
      "type/prefer-return-this-type": "warn"
    },
    "typeAware": true
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/prefer_return_this_type.rs#L5)
