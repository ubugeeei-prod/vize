---
title: "Vue Rules: Directive Validity"
---

# Vue Rules: Directive Validity

Required arguments and valid expressions for built-in Vue directives.
See [all Vue rules](./vue.md) for the complete reference and [Vue Rule Options](./options-vue.md)
for configurable settings.

## `vue/valid-v-bind`

Reports `v-bind` with no argument and no object expression, and an empty `:`
shorthand. A Vue 3.4 same-name shorthand (`:loading`) is valid.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <div v-bind></div>
  <div :></div>
</template>
```

Good:

```vue
<template>
  <div :class="panelClass"></div>
  <div v-bind="{ class: panelClass }"></div>
  <div :loading></div>
</template>
```

## `vue/valid-v-else`

Reports `v-else` with an expression, `v-else` combined with `v-if` on the
same element, and `v-else` that does not follow `v-if` or `v-else-if`.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <div v-else="ready"></div>
  <div v-else v-if="ready"></div>
  <div v-else></div>
</template>
```

Good:

```vue
<template>
  <div v-if="ready"></div>
  <div v-else></div>
</template>
```

## `vue/valid-v-for`

Reports `v-for` with no expression, an empty expression, or a modifier.
The expression has to use `in` or `of`.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <div v-for></div>
  <div v-for=""></div>
  <div v-for.stop="item in items"></div>
</template>
```

Good:

```vue
<template>
  <div v-for="item in items" :key="item.id"></div>
  <div v-for="(item, index) of items" :key="index"></div>
</template>
```

## `vue/valid-v-if`

Reports `v-if` with no expression, an empty expression, or `v-if` combined
with `v-else` / `v-else-if` on the same element.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <div v-if></div>
  <div v-if=""></div>
  <div v-if="ready" v-else></div>
</template>
```

Good:

```vue
<template>
  <div v-if="ready"></div>
  <div v-if="count > 0"></div>
</template>
```

## `vue/valid-v-memo`

Reports `v-memo` without a dependency-array expression.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <div v-memo></div>
</template>
```

Good:

```vue
<template>
  <div v-memo="[valueA, valueB]">{{ label }}</div>
</template>
```

## `vue/valid-v-model`

Reports `v-model` without an expression, and `v-model` on an element that
cannot host it. `input`, `select`, `textarea`, and components can.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <div v-model="value"></div>
  <input v-model />
</template>
```

Good:

```vue
<template>
  <input v-model="value" />
  <select v-model="selected"></select>
  <textarea v-model="text"></textarea>
  <MyInput v-model="value" />
</template>
```

## `vue/valid-v-on`

Reports `v-on` with no event name and no handler. Object syntax
(`v-on="{ click: onClick }"`) is a handler without one event name, and it is
valid.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <div v-on></div>
  <div @></div>
  <div @click></div>
</template>
```

Good:

```vue
<template>
  <div @click="handleClick"></div>
  <div v-on="{ click: handleClick }"></div>
</template>
```

## `vue/valid-v-show`

Reports `v-show` without an expression, and `v-show` on `<template>`.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <div v-show></div>
  <template v-show="ready"><div></div></template>
</template>
```

Good:

```vue
<template>
  <div v-show="ready"></div>
  <div v-show="count > 0"></div>
</template>
```

## `vue/valid-v-slot`

Reports `v-slot` on a native element, two slot directives on the same
component, and two named slots on one `<template>`.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <div v-slot:header></div>
  <MyComponent v-slot v-slot:header />
  <template v-slot:header v-slot:footer />
</template>
```

Good:

```vue
<template>
  <MyComponent v-slot="{ item }">{{ item }}</MyComponent>
  <MyComponent>
    <template #header>Header</template>
  </MyComponent>
</template>
```
