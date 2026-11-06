---
title: "ecosystem/vue-test-utils-no-html-snapshot"
description: "避免在 Vue Test Utils 测试中对 wrapper.html() 生成快照。"
---

# `ecosystem/vue-test-utils-no-html-snapshot`

避免在 Vue Test Utils 测试中对 wrapper.html() 生成快照。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `ecosystem` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

断言对整个 wrapper HTML 生成快照，而不是检查预期行为。

```vue annotate="remove:2"
<script setup lang="ts">
expect(wrapper.html()).toMatchSnapshot();
</script>
```

## 正确示例

断言检查渲染文本是否包含 Saved。

```vue annotate="add:2"
<script setup lang="ts">
expect(wrapper.text()).toContain("Saved");
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "ecosystem/vue-test-utils-no-html-snapshot": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [生态系统与 Nuxt 规则](../ecosystem.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_test_utils_no_html_snapshot.rs#L15)
