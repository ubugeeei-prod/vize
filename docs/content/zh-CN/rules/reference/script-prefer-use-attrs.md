---
title: "script/prefer-use-attrs"
description: "建议使用 useAttrs() 而不是 context.attrs。"
---

# `script/prefer-use-attrs`

建议使用 useAttrs() 而不是 context.attrs。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

`setup` 通过解构上下文参数获取 `attrs`，规则要求改用 Composition API 辅助函数。

```vue annotate="remove:2"
<script lang="ts">
export default { setup(_props, { attrs }) { console.log(attrs.class); } };
</script>
```

## 正确示例

`useAttrs()` 在 setup 内提供 `attrs`，保留 `attrs.class` 读取，不依赖第二个 setup 参数。

```vue annotate="add:2,3"
<script lang="ts">
import { useAttrs } from "vue";
export default { setup() { const attrs = useAttrs(); console.log(attrs.class); } };
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/prefer-use-attrs": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_attrs.rs#L44)
