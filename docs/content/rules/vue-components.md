---
title: "Vue Rules: Components and Props"
---

# Vue Rules: Components and Props

Component naming, registration, dynamic components, and prop ownership.
See [all Vue rules](./vue.md) for the complete reference and [Vue Rule Options](./options-vue.md)
for configurable settings.

## `vue/no-mutating-props`

Reports writes to props. The owning component should update the value through an event or a model
binding.

Default severity: `error`\
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<script setup lang="ts">
const props = defineProps<{ count: number }>();

props.count++;
</script>
```

Good:

```vue
<script setup lang="ts">
const props = defineProps<{ count: number }>();
const emit = defineEmits<{ "update:count": [value: number] }>();

function increment() {
  emit("update:count", props.count + 1);
}
</script>
```

## `vue/no-unused-components`

Reports locally registered components that never appear in the template.

Default severity: `warning`\
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<script setup lang="ts">
import UserAvatar from "./UserAvatar.vue";
</script>

<template>
  <p>{{ user.name }}</p>
</template>
```

Good:

```vue
<script setup lang="ts">
import UserAvatar from "./UserAvatar.vue";
</script>

<template>
  <UserAvatar :user="user" />
</template>
```

## `vue/no-unused-properties`

Reports props declared through `defineProps` that are not used by the component.

Default severity: `warning`\
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<script setup lang="ts">
defineProps<{ title: string; description: string }>();
</script>

<template>
  <h1>{{ title }}</h1>
</template>
```

Good:

```vue
<script setup lang="ts">
defineProps<{ title: string; description: string }>();
</script>

<template>
  <h1>{{ title }}</h1>
  <p>{{ description }}</p>
</template>
```

## `vue/require-component-is`

Reports `<component>` without an `is` binding.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <component />
</template>
```

Good:

```vue
<template>
  <component :is="currentComponent" />
</template>
```

## `vue/component-definition-name-casing`

Reports component filenames that are neither PascalCase nor kebab-case.
`MyComponent.vue` and `my-component.vue` are both accepted. `index.vue` and
`App.vue` are accepted too.

Default severity: `warning`\
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```text
myComponent.vue
my-Component.vue
```

Good:

```text
MyComponent.vue
my-component.vue
index.vue
App.vue
```

## `vue/component-name-in-template-casing`

Enforces component name casing in templates. The default is PascalCase.
Built-ins such as `<slot>` are left alone.

Default severity: `warning`\
Presets: `nuxt`, `opinionated`

Bad:

```vue
<template>
  <my-component />
  <myComponent />
</template>
```

Good:

```vue
<template>
  <MyComponent />
  <RouterView />
  <slot />
</template>
```

## `vue/multi-word-component-names`

Requires the component filename to contain more than one word, so it cannot
collide with a present or future HTML element. `App.vue` is the exception.
Names used inside the template are not what this rule checks.

Default severity: `error`\
Presets: `essential`, `nuxt`, `opinionated`

Bad:

```text
Item.vue
Table.vue
```

Good:

```text
TodoItem.vue
DataTable.vue
App.vue
```

## `vue/no-non-component-keep-alive-child`

Reports a plain element directly below `<KeepAlive>`. Vue caches component
vnodes, so a native wrapper leaves the component uncached. A wrapper whose
only directive is `v-show` is ignored. `v-show` together with something that
changes identity, such as `:key`, is still reported.

Default severity: `warning`\
Presets: none (opt-in)

Bad:

```vue
<template>
  <KeepAlive>
    <div v-if="ready">
      <UserCard />
    </div>
  </KeepAlive>
</template>
```

Good:

```vue
<template>
  <KeepAlive>
    <UserCard v-if="ready" />
  </KeepAlive>
  <KeepAlive>
    <div v-show="opened">
      <UserCard />
    </div>
  </KeepAlive>
</template>
```

## `vue/no-reserved-component-names`

Reports an explicit component name that is an HTML element, an SVG element,
or a Vue built-in. It reads the Options API `name` and
`defineOptions({ name })`. It does not read the filename, and using
`<Transition>` or `<KeepAlive>` in a template is fine.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<script>
export default {
  name: "button",
};
</script>
```

```vue
<script setup lang="ts">
defineOptions({ name: "svg" });
</script>
```

Good:

```vue
<script setup lang="ts">
defineOptions({ name: "AppButton" });
</script>

<template>
  <Transition>
    <AppButton />
  </Transition>
</template>
```

## `vue/require-component-registration`

Reports a component used in the template that was not imported in
`<script setup>` and was not registered. Built-ins such as `<component>` and
`<Transition>` are ignored.

Default severity: `warning`\
Presets: `opinionated`

Bad:

```vue
<script setup lang="ts">
// MyButton is never imported.
</script>

<template>
  <MyButton>Save</MyButton>
</template>
```

Good:

```vue
<script setup lang="ts">
import MyButton from "./MyButton.vue";
</script>

<template>
  <MyButton>Save</MyButton>
</template>
```
