---
title: UI 样式
---

<!-- Generated translation; source: guide/ui-styles.md -->

# UI 样式

`@vizejs/ui` 组件无需视觉样式表即可工作。要使用 Vize 的可选样式，请导入基础样式、一个调色板，以及页面所用的组件：

```ts
import "@vizejs/ui/base.css";
import "@vizejs/ui/theme-preset-paper.css";
import "@vizejs/ui/component-button.css";
import "@vizejs/ui/component-input.css";
import "@vizejs/ui/component-textarea.css";
import "@vizejs/ui/component-checkbox.css";
import "@vizejs/ui/component-switch.css";
import "@vizejs/ui/component-dialog.css";
import "@vizejs/ui/component-card.css";
import "@vizejs/ui/component-badge.css";
import "@vizejs/ui/component-alert.css";
import "@vizejs/ui/component-tooltip.css";
// Add these only when their low-level behavior is used without the JS entry:
// import "@vizejs/ui/component-progress-bar.css";
// import "@vizejs/ui/component-scroll-area.css";
// import "@vizejs/ui/motion.css";
```

```vue
<template>
  <main data-vize-theme="paper">
    <Button>Save changes</Button>
  </main>
</template>
```

`base.css` 提供语义令牌、密度和 forced-colors 策略；它是现有 `theme.css` 导出的别名。它不会重置应用程序元素，也不会改变未加样式的 Vize 组件。组件文件是纯 CSS 资源，不会改变 JavaScript 或 Vue API。单独导入某个组件不会添加其他任何组件的视觉规则。

Input、Textarea、Checkbox 和 Switch 各自拥有独立的 CSS 导出。文本字段保留原生的编辑和缩放行为；Checkbox 区分选中与混合状态，而 Switch 在状态变化时移动其滑块。这些细微的状态过渡在 `prefers-reduced-motion` 下会停止。键盘用户始终可以看到焦点，forced-colors 模式会保留原生复选框标记。组件文件会在每个主题边界处重置其局部覆盖，因此嵌套在 Signal 页面中的 Paper 表单仍会保持 Paper 的字体与比例。

Card、Badge、Alert 和 Tooltip 同样各有独立的视觉文件。Card 使用其 `variant`、`density` 和 `tone` 钩子；Badge 区分标签、计数和状态文本；Alert 遵循其实时区域变体，且不会添加关闭按钮。Tooltip 保持触发元素的键盘焦点可见，并且只在测量出浮动位置之后才开始其简短的入场动画。静态 Card 表面不带动画。Badge 的色调变化以及 Alert/Tooltip 的入场动画在 `prefers-reduced-motion` 下会停止；forced-colors 模式会恢复系统边界。这些样式会在嵌套的主题边界处重置，包括 Shadow DOM 宿主。

现有的 ProgressBar 和 ScrollArea 的结构与动效方案也以独立的纯 CSS 文件发布。它们的 JavaScript 入口已经为必要的行为引入了旧版聚合样式表。在不使用 JavaScript 入口而使用 CSS 钩子时，或需要显式控制样式表时，请导入独立文件；避免在同一页面中同时导入两种路径。

| 预设      | 特点                                         |
| --------- | -------------------------------------------- |
| `paper`   | 温暖的纸张、墨色、细边框和方正的控件         |
| `signal`  | 紧凑的石墨色表面，边缘清晰，几乎没有层次深度 |
| `atelier` | 安静的工作室中性色，搭配一个克制的强调色     |

要提供样式切换器，请将上面的单个预设导入替换为你想提供的预设。每个样式表只在其自身的作用域内生效：

```ts
import "@vizejs/ui/theme-preset-paper.css";
import "@vizejs/ui/theme-preset-signal.css";
import "@vizejs/ui/theme-preset-atelier.css";

document.documentElement.dataset.vizeTheme = "signal";
```

现有的 `midnight`、`play`、`high-contrast` 和 `headless` 预设仍然可用。当对话框或工具提示被传送到文档 body 时，请在 `<html>` 上设置 `data-vize-theme`，使浮动内容继承与页面相同的调色板。对于已保存的偏好，`@vizejs/ui/theme-scope` 提供了一个绘制前运行的引导脚本。旧的 `theme.css`、`theme-preset-*.css` 和 `style.css` 导入仍可继续使用；避免将 `style.css` 与 `base.css` 一起导入，因为它已经包含基础样式和所有旧版预设。

Button 具有简短的悬停、按下和焦点反馈。配合 `dialog.css`，Dialog 会为出现过程添加动画，并有 200ms 的退出动画。关闭时会立即解除焦点限制、外部 inert 和滚动锁定；正在退出的面板在动画结束前保持 inert，并对辅助技术隐藏。Headless Dialog 仍会立即卸载；当匹配 `prefers-reduced-motion: reduce` 时，带样式的 Dialog 也会如此。两个视觉文件在 forced-colors 模式下都保留清晰的边界。位于 Vize 级联层之外的应用程序 CSS 可以覆盖任何规则。
