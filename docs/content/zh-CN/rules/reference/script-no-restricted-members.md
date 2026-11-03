---
title: "script/no-restricted-members"
description: "禁止项目配置的 object.property 成员访问。"
---

# `script/no-restricted-members`

禁止项目配置的 object.property 成员访问。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | 无 | 不支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

此示例配置 window.localStorage。规则没有默认拒绝列表；仅启用规则不会报告成员。

## 错误示例

在 `ruleOptions` 中配置 `{ object: "window", property: "localStorage" }` 后，`window.localStorage` 访问了禁止的对象与成员组合。规则没有默认禁止成员。

```vue annotate="remove:2"
<script setup lang="ts">
const token = window.localStorage.getItem("token");
</script>
```

## 正确示例

`authStorage.read("token")` 将读取交给应用存储辅助函数，不再访问配置禁止的 `window.localStorage` 成员。

```vue annotate="add:2"
<script setup lang="ts">
const token = authStorage.read("token");
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "script/no-restricted-members": "error"
      },
      "ruleOptions": {
        "script/no-restricted-members": {
          "members": [
            {
              "object": "window",
              "property": "localStorage"
            }
          ]
        }
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [规则选项](/rules/options.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_restricted_members.rs#L53)
