---
title: 蒸汽规则
---

<!-- Generated translation; source: rules/vapor.md -->

# 蒸汽统治

这些规则涵盖了面向蒸汽组件和应用的模板约束。组合API和
脚本级的蒸汽指导存在于[类型和脚本规则](./type-and-script.md)。

## `vapor/no-vue-lifecycle-events`

报告每个元素的生命周期事件，如`@vue:mounted`。

默认严重程度：`error`
预设：`happy-path`，`nuxt`，`opinionated`

缺点：

```vue
<template>
  <input @vue:mounted="focusInput" />
</template>
```

好：

```vue
<script setup lang="ts" vapor>
const input = useTemplateRef<HTMLInputElement>("input");

onMounted(() => {
  input.value?.focus();
});
</script>

<template>
  <input ref="input" />
</template>
```

## `vapor/require-vapor-attribute`

建议在预设预期为蒸汽兼容组件时，给`<script setup>`添加`vapor`。

默认严重程度：`warning`
预设：`nuxt`，`opinionated`

缺点：

```vue
<script setup lang="ts">
const count = ref(0);
</script>
```

好：

```vue
<script setup lang="ts" vapor>
const count = ref(0);
</script>
```

## `vapor/no-inline-template`

报告已废弃`inline-template`属性。

默认严重程度：`error`
预设：`nuxt`，`opinionated`

缺点：

```vue
<template>
  <LegacyCard inline-template>
    <p>Profile</p>
  </LegacyCard>
</template>
```

好：

```vue
<template>
  <LegacyCard>
    <template #default>
      <p>Profile</p>
    </template>
  </LegacyCard>
</template>
```

## `vapor/prefer-static-class`

报告动态`:class`绑定，其值为静态字符串文字。

默认严重程度：`warning`
预设：`nuxt`，`opinionated`

缺点：

```vue
<template>
  <section :class="'panel panel-primary'">Profile</section>
</template>
```

好：

```vue
<template>
  <section class="panel panel-primary">Profile</section>
</template>
```
