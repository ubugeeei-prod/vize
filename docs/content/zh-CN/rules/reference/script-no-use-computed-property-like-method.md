---
title: "script/no-use-computed-property-like-method"
description: "禁止像方法一样调用 Options API 计算属性。"
---

# `script/no-use-computed-property-like-method`

禁止像方法一样调用 Options API 计算属性。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

`this.total()` 调用了计算 getter 暴露的值；getter 返回不可调用的 `3`。

```vue annotate="remove:2"
<script lang="ts">
export default { computed: { total() { return 3; } }, methods: { log() { console.log(this.total()); } } };
</script>
```

## 正确示例

`this.total` 不加调用括号，直接读取计算值，使 `log` 打印派生数字。

```vue annotate="add:2"
<script lang="ts">
export default { computed: { total() { return 3; } }, methods: { log() { console.log(this.total); } } };
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-use-computed-property-like-method": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_use_computed_property_like_method.rs#L44)
