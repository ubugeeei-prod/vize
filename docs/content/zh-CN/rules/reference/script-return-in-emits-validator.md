---
title: "script/return-in-emits-validator"
description: "要求每个 Options API emits 校验函数返回值。"
---

# `script/return-in-emits-validator`

要求每个 Options API emits 校验函数返回值。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

为当前支持的 SFC 筛选使用块函数体箭头函数。底层校验器也处理方法简写，但当前 SFC 前置筛选无法可靠分派该形式。

## 错误示例

`submit` 校验函数记录载荷，却不返回校验结果，因此块函数体产生 undefined。

```vue annotate="remove:2"
<script lang="ts">
export default { emits: { submit: (payload: unknown) => { console.log(payload); } } };
</script>
```

## 正确示例

`return payload != null` 为提交载荷提供布尔校验结果，避免结束时没有返回值。

```vue annotate="add:2"
<script lang="ts">
export default { emits: { submit: (payload: unknown) => { return payload != null; } } };
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/return-in-emits-validator": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/return_in_emits_validator.rs#L61)
