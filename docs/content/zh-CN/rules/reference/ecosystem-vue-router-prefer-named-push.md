---
title: "ecosystem/vue-router-prefer-named-push"
description: "Vue Router 编程式导航优先使用具名路由对象。"
---

# `ecosystem/vue-router-prefer-named-push`

Vue Router 编程式导航优先使用具名路由对象。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `ecosystem` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

router.push 接收与当前 URL 写法绑定的路径字符串。

```vue annotate="remove:2"
<script setup lang="ts">
router.push("/settings");
</script>
```

## 正确示例

router.push 接收具有稳定 settings 名称的路由对象。

```vue annotate="add:2"
<script setup lang="ts">
router.push({ name: "settings" });
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "ecosystem/vue-router-prefer-named-push": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [生态系统与 Nuxt 规则](../ecosystem.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_router_prefer_named_push.rs#L19)
