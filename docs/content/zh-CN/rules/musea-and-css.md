---
title: Musea 与 CSS 规则
---

<!-- Generated translation; source: rules/musea-and-css.md -->

# 博物馆与CSS规则

Musea规则验证`<art>`和`<variant>`块。CSS 规则检查风格内容并推荐
保持组件样式主题化、可预测性，并与Vue和Vapor 兼容的模式。

## `musea/require-title`

要求每个艺术文件都必须提供显示标题。标题可以来自`<art title="...">`，
`defineArt("./Button.vue", { title: "..." })`，或者`defineArt`组件源的备用。

默认严重程度：`error`

缺点：

```vue
<art component="./Button.vue">
  <variant name="primary" />
</art>
```

好：

```vue
<script setup>
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="primary" />
</art>
```

## `musea/require-component`

要求每个艺术文件都为它所记录的组件命名。更喜欢`defineArt("./Button.vue", ...)`;
`<art component="...">`仍然支持兼容性。

默认严重程度：`warning`

缺点：

```vue
<art title="Button">
  <variant name="primary" />
</art>
```

好：

```vue
<script setup>
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="primary" />
</art>
```

## `musea/valid-variant`

要求`<variant>`块必须有有效的`name`。

默认严重程度：`error`

缺点：

```vue
<art title="Button" component="./Button.vue">
  <variant />
</art>
```

好：

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

## `musea/unique-variant-names`

要求变体名称在同一艺术区块内是唯一的。

默认严重程度：`error`

缺点：

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary" />
  <variant name="primary" />
</art>
```

好：

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary" />
  <variant name="secondary" />
</art>
```

## `musea/no-empty-variant`

报告空变体，这些变体不记录道具、槽位或视觉状态。

默认严重程度：`warning`

缺点：

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

好：

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary">
    <Button tone="primary">Save</Button>
  </variant>
</art>
```

## `musea/prefer-design-tokens`

在 Musea 示例中，更倾向于使用设计令牌 CSS 变量，而非硬编码的原始值。

默认严重程度：`warning`

缺点：

```vue
<art title="Button" component="./Button.vue">
  <variant name="danger">
    <Button style="color: #d00">Delete</Button>
  </variant>
</art>
```

好：

```vue
<art title="Button" component="./Button.vue">
  <variant name="danger">
    <Button class="danger">Delete</Button>
  </variant>
</art>

<style scoped>
.danger {
  color: var(--color-danger-text);
}
</style>
```

## `css/no-important`

这让我却步，打消`!important`。

默认严重程度：`warning`

缺点：

```vue
<style scoped>
.button {
  color: red !important;
}
</style>
```

好：

```vue
<style scoped>
.button {
  color: var(--button-color);
}
</style>
```

## `css/no-hardcoded-values`

建议用CSS变量代替硬编码的颜色、间距或大小值。

默认严重程度：`warning`

缺点：

```vue
<style scoped>
.button {
  padding: 12px 16px;
  color: #174ea6;
}
</style>
```

好：

```vue
<style scoped>
.button {
  padding: var(--space-3) var(--space-4);
  color: var(--color-action-text);
}
</style>
```

## `css/no-id-selectors`

在组件样式中不鼓励使用ID选择器，因为它们难以覆盖和重复使用。

默认严重程度：`warning`

缺点：

```vue
<style scoped>
#submit {
  font-weight: 600;
}
</style>
```

好：

```vue
<style scoped>
.submit {
  font-weight: 600;
}
</style>
```

## `css/no-display-none`

建议使用 Vue 可视化原语，而不是用 CSS 隐藏组件分支。

默认严重程度：`warning`

缺点：

```vue
<template>
  <p class="message">Saved</p>
</template>

<style scoped>
.message {
  display: none;
}
</style>
```

好：

```vue
<template>
  <p v-show="isSaved" class="message">Saved</p>
</template>
```

## `css/no-v-bind-performance`

提醒热样式中 CSS `v-bind()` 的运行成本。

默认严重程度：`warning`

缺点：

```vue
<style scoped>
.card {
  transform: translateX(v-bind(offset));
}
</style>
```

好：

```vue
<template>
  <article :style="{ transform: `translateX(${offset}px)` }" class="card" />
</template>
```

## `css/prefer-logical-properties`

推荐国际化布局的逻辑属性。

默认严重程度：`warning`

缺点：

```vue
<style scoped>
.panel {
  margin-left: 1rem;
}
</style>
```

好：

```vue
<style scoped>
.panel {
  margin-inline-start: 1rem;
}
</style>
```

## `css/prefer-slotted`

建议在设计槽位内容时要`::v-slotted()`。

默认严重程度：`warning`

缺点：

```vue
<style scoped>
.content h2 {
  margin-block: 0;
}
</style>
```

好：

```vue
<style scoped>
::v-slotted(h2) {
  margin-block: 0;
}
</style>
```

## `css/require-font-display`

`@font-face`声明中需要`font-display`。

默认严重程度：`warning`

缺点：

```vue
<style>
@font-face {
  font-family: "Inter";
  src: url("/inter.woff2") format("woff2");
}
</style>
```

好：

```vue
<style>
@font-face {
  font-family: "Inter";
  src: url("/inter.woff2") format("woff2");
  font-display: swap;
}
</style>
```

## 额外的CSS规则

`css/no-utility-classes`警告不要在组件样式中实现实用类。默认：
`warning`。

`css/prefer-nested-selectors`建议后代选择器使用CSS嵌套。默认值：`warning`。
