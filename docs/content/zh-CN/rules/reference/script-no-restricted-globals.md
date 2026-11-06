---
title: "script/no-restricted-globals"
description: "禁止直接引用必须通过类型化包装器访问的运行时环境全局变量。"
---

# `script/no-restricted-globals`

禁止直接引用必须通过类型化包装器访问的运行时环境全局变量。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | 无 | 不支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

示例直接读取默认受限全局变量 `process`、`localStorage` 和 `sessionStorage`，绕过项目显式的配置和存储辅助函数。

```vue annotate="remove:2,3,4"
<script setup lang="ts">
const flag = process.env.FEATURE_FLAG
const token = localStorage.getItem('auth.token')
sessionStorage.setItem('view.scroll', String(window.scrollY))
</script>
```

## 正确示例

`useFeatureFlag`、`authStorage.read` 和 `viewStorage.write` 移除了这些直接引用。剩余的 `window.scrollY` 不是规则的默认限制；SSR 安全是独立问题。

```vue annotate="add:2,3,4,5,6,7"
<script setup lang="ts">
// Use a typed config helper that distinguishes server vs. client.
const flag = useFeatureFlag('FEATURE_FLAG')

// Use a typed wrapper that scopes keys and handles SSR / disabled storage.
const token = authStorage.read('auth.token')
viewStorage.write('view.scroll', String(window.scrollY))
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-restricted-globals": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [规则选项](/rules/options.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_restricted_globals.rs#L57)
