---
title: 无障碍规则
---

<!-- Generated translation; source: rules/accessibility.md -->

# 无障碍规则

无障碍规则是Patina单文件模板规则。他们能追上难以追上的加价
与辅助技术或键盘导航结合使用。

## `a11y/img-alt`

需要在`<img>`上有个`alt`属性。

默认严重程度：`warning`
预设：`happy-path`，`nuxt`，`opinionated`

缺点：

```vue
<template>
  <img src="/avatar.png" />
</template>
```

好：

```vue
<template>
  <img src="/avatar.png" alt="User avatar" />
</template>
```

## `a11y/alt-text`

需要替代文本的媒体元素。

默认严重程度：`warning`
预设：`happy-path`，`nuxt`，`opinionated`

缺点：

```vue
<template>
  <input type="image" src="/submit.png" />
</template>
```

好：

```vue
<template>
  <input type="image" src="/submit.png" alt="Submit" />
</template>
```

## `a11y/click-events-have-key-events`

报告在没有键盘处理器时，会点击非原生交互元素的处理程序。

默认严重程度：`warning`
预设：`happy-path`，`nuxt`，`opinionated`

缺点：

```vue
<template>
  <div role="button" @click="save">Save</div>
</template>
```

好：

```vue
<template>
  <button type="button" @click="save">Save</button>
</template>
```

## `a11y/interactive-supports-focus`

需要带有交互角色的元素才能被聚焦。

默认严重程度：`warning`
预设：`happy-path`，`nuxt`，`opinionated`

缺点：

```vue
<template>
  <span role="button" @click="open">Open</span>
</template>
```

好：

```vue
<template>
  <button type="button" @click="open">Open</button>
</template>
```

## `a11y/label-has-for`

要求标签必须与表单控制关联。

默认严重程度：`warning`
预设：`happy-path`，`nuxt`，`opinionated`

缺点：

```vue
<template>
  <label>Email</label>
  <input id="email" />
</template>
```

好：

```vue
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

## `a11y/form-control-has-label`

要求控件具有可见或程序化标签。

默认严重程度：`warning`
预设：`happy-path`，`nuxt`，`opinionated`

缺点：

```vue
<template>
  <input type="search" />
</template>
```

好：

```vue
<template>
  <label>
    Search
    <input type="search" />
  </label>
</template>
```

## `a11y/no-aria-hidden-on-focusable`

报告了可聚焦的元素，隐藏在辅助技术之外。

默认严重程度：`error`
预设：`happy-path`，`nuxt`，`opinionated`

缺点：

```vue
<template>
  <button aria-hidden="true" @click="close">Close</button>
</template>
```

好：

```vue
<template>
  <button aria-label="Close" @click="close">Close</button>
</template>
```

## `a11y/no-static-element-interactions`

报告静态元素上的鼠标或键盘处理器。

默认严重程度：`warning`
预设：`happy-path`，`nuxt`，`opinionated`

缺点：

```vue
<template>
  <section @click="select">Select</section>
</template>
```

好：

```vue
<template>
  <button type="button" @click="select">Select</button>
</template>
```

## `a11y/tabindex-no-positive`

报告正值`tabindex`是因为它们创建了难以预测的自定义制表顺序。

默认严重程度：`warning`
预设：`happy-path`，`nuxt`，`opinionated`

缺点：

```vue
<template>
  <button tabindex="3">Save</button>
</template>
```

好：

```vue
<template>
  <button>Save</button>
</template>
```

## `a11y/anchor-is-valid`

要求锚点必须有有效的链路目标。
静态`href`值在方案归一化后检查，因此`JaVaScRiPt:`并用HTML解码
`java&#x0A;script:`中的控制字符仍然会被报告，同时类似的非匹配方案
允许留下。

默认严重程度：`warning`
预设：`happy-path`，`nuxt`，`opinionated`

缺点：

```vue
<template>
  <a href="#" @click="open">Open</a>
  <a href="JaVaScRiPt:void(0)">Open</a>
</template>
```

好：

```vue
<template>
  <button type="button" @click="open">Open</button>
  <a href="/docs/javascript:void">Docs</a>
</template>
```

## 额外的无障碍规则

`a11y/anchor-has-content`要求锚点元素包含可访问的内容。默认值：`warning`。
预设：`happy-path`、`nuxt`、`opinionated`。

`a11y/aria-props`禁止无效的ARIA属性。默认值：`error`。预设：`happy-path`，
`nuxt`，`opinionated`。

`a11y/aria-role`要求有效的、非抽象的ARIA角色。默认值：`error`。预设：`happy-path`，
`nuxt`，`opinionated`。

`a11y/aria-unsupported-elements`禁止在不支持ARIA属性的元素上添加属性。
默认值：`error`。预设：`happy-path`、`nuxt`、`opinionated`。

`a11y/heading-has-content`需要标题元素以包含易于理解的内容。默认值：`warning`。
预设：`happy-path`、`nuxt`、`opinionated`。

`a11y/heading-levels`不允许跳过航向等级。默认值：`warning`。预设：`nuxt`，
`opinionated`。

`a11y/iframe-has-title`要求`<iframe>`有`title`。默认值：`warning`。预设：
`happy-path`，`nuxt`，`opinionated`。

`a11y/landmark-roles`验证了具有里程碑意义的角色定位和独特性。默认值：`warning`。
预设：`nuxt`，`opinionated`。

`a11y/media-has-caption`需要为媒体元素提供说明。默认值：`warning`。预设：
`happy-path`，`nuxt`，`opinionated`。

`a11y/mouse-events-have-key-events`需要对焦和使用鼠标操作时的模糊操作。
默认值：`warning`。预设：`happy-path`、`nuxt`、`opinionated`。

`a11y/no-access-key`不允许`accesskey`属性。默认：`warning`。预设：
`happy-path`，`nuxt`，`opinionated`。

`a11y/no-autofocus`不允许`autofocus`。默认：`warning`。预设：`happy-path`，`nuxt`，
`opinionated`。

`a11y/no-distracting-elements`禁止`<marquee>`和`<blink>`等干扰性元素。
默认值：`warning`。预设：`happy-path`、`nuxt`、`opinionated`。

`a11y/no-i-for-icon`不鼓励将`<i>`仅作为图标元素使用。默认：`warning`。预设：
`happy-path`，`nuxt`，`opinionated`。

`a11y/no-redundant-roles`禁止重复本地语义的ARIA角色。默认：
`warning`。预设：`happy-path`、`nuxt`、`opinionated`。

`a11y/no-refer-to-non-existent-id`报告了ARIA对缺失身份证的提及。默认值：`warning`。
预设：`happy-path`、`nuxt`、`opinionated`。

`a11y/no-role-presentation-on-focusable`不允许`role="presentation"`或`role="none"` on
可聚焦元素。默认值：`error`。预设：`happy-path`、`nuxt`、`opinionated`。

`a11y/placeholder-label-option`需要在占位符的`<option>`值上被禁用或隐藏。
默认：`warning`。预设：`nuxt`，`opinionated`。

`a11y/role-has-required-aria-props`要求角色包含其必需的ARIA属性。
默认值：`warning`。预设：`happy-path`、`nuxt`、`opinionated`。

`a11y/use-list`建议为项目符号状文本提供列表元素。默认值：`warning`。预设：`nuxt`，
`opinionated`。
