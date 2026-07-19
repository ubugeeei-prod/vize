---
title: 生态系统规则
---

<!-- Generated translation; source: rules/ecosystem.md -->

# 生态系统规则

这些规则涵盖了Nuxt、Vue Router、Pinia、vue-i18n、Vue测试工具和Void Vue等约定。

生态系统规则由`ecosystem`预设启用。主机在使用时也可以以名称启用
`incremental`;他们不属于`happy-path`、`nuxt`或`opinionated`。

当 LSP 启用编辑器生态系统辅助工具时，Vize 还会添加 Vue 路由器路由名称
完成、文件路由参数补全及`useRoute().params`诊断，Vue I18n密钥
完成、工作区JSON密钥验证，以及静态`t()`/`$t()`调用的嵌入预览。

## `ecosystem/router-link-require-to`

需要`<RouterLink>`、`<router-link>`、`<NuxtLink>`和`<nuxt-link>`的`to`或`:to`。

默认严重程度：`error`
预设：`ecosystem`

缺点：

```vue
<template>
  <RouterLink>Settings</RouterLink>
</template>
```

好：

```vue
<template>
  <RouterLink :to="{ name: 'settings' }">Settings</RouterLink>
</template>
```

## `ecosystem/vue-router-prefer-named-link`

在类似RouterLink的组件中，对静态内部路径字符串进行警告。命名路由对象保留Vue
路由器的类型路由和编辑器的补全都围绕路线名称和参数展开。

默认严重程度：`warning`
预设：`ecosystem`

缺点：

```vue
<template>
  <RouterLink to="/settings">Settings</RouterLink>
</template>
```

好：

```vue
<template>
  <RouterLink :to="{ name: 'settings' }">Settings</RouterLink>
</template>
```

## `ecosystem/vue-router-prefer-named-push`

`router.push("/path")`警告，`router.replace("/path")`，并用静态`path`引导物体。

默认严重程度：`warning`
预设：`ecosystem`

缺点：

```ts
router.push("/settings");
```

好：

```ts
router.push({ name: "settings" });
```

## `ecosystem/nuxt-prefer-nuxt-link`

在面向 Nuxt 的代码中警告内部`<a href="/...">`链接。外部链接、下载及
`target="_blank"`依然是普通的锚点。

默认严重程度：`warning`
预设：`ecosystem`

缺点：

```vue
<template>
  <a href="/settings">Settings</a>
</template>
```

好：

```vue
<template>
  <NuxtLink to="/settings">Settings</NuxtLink>
</template>
```

## `ecosystem/pinia-prefer-store-to-refs`

当Pinia商店被直接拆解时会发出警告。使用`storeToRefs()`表示状态和获取指标，并且
把操作都保留在商店实例上。

默认严重程度：`warning`
预设：`ecosystem`

缺点：

```ts
const { name } = useUserStore();
```

好：

```ts
const store = useUserStore();
const { name } = storeToRefs(store);
```

## `ecosystem/vue-i18n-no-missing-key`

当同一键缺少静态`$t()`、`$te()`、`$tm()`、`t()`、`te()`或`tm()`键时，会发出警告
SFC的本地`<i18n lang="json">`栋。

默认严重程度：`warning`
预设：`ecosystem`

缺点：

```vue
<template>{{ $t("auth.missing") }}</template>

<i18n lang="json">
{ "en": { "auth": { "login": "Log in" } } }
</i18n>
```

好：

```vue
<template>{{ $t("auth.login") }}</template>

<i18n lang="json">
{ "en": { "auth": { "login": "Log in" } } }
</i18n>
```

## `ecosystem/void-link-require-href`

需要在Void Vue上`href`或`:href` `<Link>`从`@void/vue`导入的组件。

默认严重程度：`error`
预设：`ecosystem`

缺点：

```vue
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link>Settings</Link>
</template>
```

好：

```vue
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/settings">Settings</Link>
</template>
```

## `ecosystem/void-link-valid-method`

对未知静态Void Vue `<Link method>`值以及仅支持GET的道具（如`prefetch`）发出警告
或者当链接使用变异方法时`reloadDocument`。

默认严重程度：`warning`
预设：`ecosystem`

缺点：

```vue
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/posts/1" method="DELETE" prefetch>Delete</Link>
</template>
```

好：

```vue
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/posts/1" method="DELETE">Delete</Link>
</template>
```

## `ecosystem/vue-test-utils-no-html-snapshot`

`expect(wrapper.html()).toMatchSnapshot()`警告。更喜欢围绕可见文字进行聚焦的断言，
属性、已发射事件或组件状态。

默认严重程度：`warning`
预设：`ecosystem`

缺点：

```ts
expect(wrapper.html()).toMatchSnapshot();
```

好：

```ts
expect(wrapper.text()).toContain("Saved");
```
