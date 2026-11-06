---
title: "vue/match-component-file-name"
description: "要求声明的组件名称与文件名匹配。"
---

# `vue/match-component-file-name`

要求声明的组件名称与文件名匹配。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

文件名也是检查的一部分。默认比较方式是将声明的组件名转换为 PascalCase 或 kebab-case，再与未作变换、去掉扩展名的文件名比较。包含多个组件定义的文件不作检查。

## 错误示例

声明的组件名 `OtherPanel` 即使经过支持的大小写转换，也不匹配 `MyPanel.vue` 去掉扩展名后的文件名。

`MyPanel.vue`

```vue annotate="remove:2"
<script setup lang="ts">
defineOptions({ name: "OtherPanel" });
</script>
```

## 正确示例

将组件命名为 `MyPanel`，使声明的名称与所在文件一致。

`MyPanel.vue`

```vue annotate="add:2"
<script setup lang="ts">
defineOptions({ name: "MyPanel" });
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/match-component-file-name": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/match_component_file_name.rs#L21)
