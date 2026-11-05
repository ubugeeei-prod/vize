---
title: "vue/use-v-on-exact"
description: "存在带按键修饰符的处理器时，要求在 `v-on` 上使用 `.exact`。"
---

# `vue/use-v-on-exact`

存在带按键修饰符的处理器时，要求在 `v-on` 上使用 `.exact`。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 处理器何时会重叠

此规则比较原生元素上同一事件的监听器。对于自定义组件事件，只有两个监听器都使用 `.native` 时才进行检查。事件名称不同或按键过滤器没有共同按键时，不会发生冲突。

系统修饰符指定必须按下的按键，因此 `@keydown.alt.shift` 也接受 Ctrl+Alt+Shift。如果另一个处理器使用 `@keydown.alt.ctrl.shift`，请为匹配范围更广的处理器添加 `.exact`，以区分这些操作。`.exact` 允许其声明的系统修饰符，并拒绝额外的 Ctrl、Shift、Alt 或 Meta 键。

多个键盘修饰符表示备选项：`@keydown.up.down` 接受上、下两个方向键中的任意一个。按下 Ctrl+下方向键时，它可能与 `@keydown.ctrl.down` 重叠。规则报告需要 `.exact` 的指令键，不包括其处理器表达式。

## 错误示例

普通点击处理器也可能在 Ctrl-click 时运行，与独立的 `.ctrl` 处理器重叠。

```vue annotate="remove:2"
<template>
  <button type="button" @click="handleClick" @click.ctrl="handleCtrlClick">
    Save
  </button>
</template>
```

## 正确示例

`.exact` 将普通点击处理器限制为未按修饰键的点击；Ctrl 专用处理器仍保持独立。

```vue annotate="add:2,3,4,5,6"
<template>
  <button
    type="button"
    @click.exact="handleClick"
    @click.ctrl="handleCtrlClick"
  >
    Save
  </button>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/use-v-on-exact": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_v_on_exact.rs#L31)
