---
title: "type/no-meaningless-void-operator"
description: "禁止对已经是 void 的表达式使用 void 运算符。"
---

# `type/no-meaningless-void-operator`

禁止对已经是 void 的表达式使用 void 运算符。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 脚本和模板中下方所示结构的类型信息。

这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。

## 错误示例

logVoid() 已经是 void，因此 void 运算符不会改变结果。

```vue annotate="remove:2,3"
<script setup lang="ts">
function logVoid(): void {}
void logVoid();
</script>
```

## 正确示例

getNumber() 返回 Promise，因此 void 是有意丢弃这个 thenable。

```vue annotate="add:2,3,4,5"
<script setup lang="ts">
function getNumber(): Promise<number> {
  return Promise.resolve(1);
}
void getNumber();
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-meaningless-void-operator": "warn"
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
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_meaningless_void_operator.rs#L5)
