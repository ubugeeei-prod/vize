---
title: "vue/scoped-event-names"
description: "建议使用 context:event 格式的作用域事件名称。"
---

# `vue/scoped-event-names`

建议使用 context:event 格式的作用域事件名称。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

`playAudio`、`pauseAudio` 和 `reloadAudio` 使用驼峰后缀表达作用域，不符合规则要求的冒号分隔事件规范。

```vue annotate="remove:3,4,5"
<template>
  <AudioPlayer
    @playAudio="play"
    @pauseAudio="pause"
    @reloadAudio="reload"
  />
</template>
```

## 正确示例

`audio:play`、`audio:pause` 和 `audio:reload` 共享显式的 `audio:` 作用域。发出事件的组件必须使用相同名称。

```vue annotate="add:3,4,5"
<template>
  <AudioPlayer
    @audio:play="play"
    @audio:pause="pause"
    @audio:reload="reload"
  />
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/scoped-event-names": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/scoped_event_names.rs#L31)
