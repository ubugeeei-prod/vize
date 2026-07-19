---
title: HTML规则
---

<!-- Generated translation; source: rules/html.md -->

# HTML规则

这些规则涵盖了 Vue 模板中的 HTML 有效性和语义标记。它们是独立于
Vue专用指令规则和无障碍规则，以便启用HTML符合性检查
或者单独解释。

## `html/id-duplication`

报告会在同一模板内重复静态ID。

默认严重程度：`error`
预设：`essential`、`happy-path`、`nuxt`、`opinionated`

缺点：

```vue
<template>
  <label for="email">Email</label>
  <input id="email" />
  <p id="email">Required</p>
</template>
```

好：

```vue
<template>
  <label for="email">Email</label>
  <input id="email" aria-describedby="email-help" />
  <p id="email-help">Required</p>
</template>
```

## `html/deprecated-element`

报告已弃用HTML元素。

默认严重程度：`warning`
预设：`happy-path`，`nuxt`，`opinionated`

缺点：

```vue
<template>
  <center>Profile</center>
</template>
```

好：

```vue
<template>
  <section class="profile">Profile</section>
</template>
```

## `html/deprecated-attr`

报告已弃用HTML属性。

默认严重程度：`warning`
预设：`happy-path`，`nuxt`，`opinionated`

缺点：

```vue
<template>
  <table border="1">
    <tr>
      <td>Total</td>
    </tr>
  </table>
</template>
```

好：

```vue
<template>
  <table class="summary">
    <tr>
      <td>Total</td>
    </tr>
  </table>
</template>
```

## `html/no-consecutive-br`

报告用于布局的连续`<br>`元素。

默认严重程度：`warning`
预设：`happy-path`，`nuxt`，`opinionated`

缺点：

```vue
<template>
  <p>First line<br /><br />Second block</p>
</template>
```

好：

```vue
<template>
  <p>First line</p>
  <p>Second block</p>
</template>
```

## `html/require-datetime`

需要机器可读的 `datetime` 值在`<time>`上。

默认严重程度：`warning`
预设：`happy-path`，`nuxt`，`opinionated`

缺点：

```vue
<template>
  <time>May 13, 2026</time>
</template>
```

好：

```vue
<template>
  <time datetime="2026-05-13">May 13, 2026</time>
</template>
```

## `html/no-duplicate-dt`

报告在同一`<dl>`内重复`<dt>`术语。

默认严重程度：`warning`
预设：`happy-path`，`nuxt`，`opinionated`

缺点：

```vue
<template>
  <dl>
    <dt>API</dt>
    <dd>Public interface</dd>
    <dt>API</dt>
    <dd>Internal service</dd>
  </dl>
</template>
```

好：

```vue
<template>
  <dl>
    <dt>API</dt>
    <dd>Public interface</dd>
    <dd>Internal service</dd>
  </dl>
</template>
```

## `html/no-empty-palpable-content`

报告空白的元素，这些元素本应暴露可见或可感知的内容。
包含文本、子内容、`aria-label`、`aria-labelledby`、`v-html`或`v-text`的元素包括
接受。

默认严重程度：`warning`
预设：`happy-path`，`nuxt`，`opinionated`

缺点：

```vue
<template>
  <p></p>
  <li></li>
  <td></td>
</template>
```

好：

```vue
<template>
  <p>Overview</p>
  <li>{{ item.label }}</li>
  <td aria-label="No value"></td>
</template>
```
