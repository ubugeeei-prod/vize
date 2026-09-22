---
title: "Vue Rules: Directive Conventions"
---

# Vue Rules: Directive Conventions

Event naming, modifier usage, directive spelling, and custom directives.
See [all Vue rules](./vue.md) for the complete reference and [Vue Rule Options](./options-vue.md)
for configurable settings.

## `vue/scoped-event-names`

Recommends `context:event` for component listeners whose names end in a
known context such as `Audio`, `Form`, or `Dialog`. A single `@playAudio`
listener is enough to trigger the rule; native element events are skipped.

Default severity: `warning`\
Presets: `nuxt`, `opinionated`

Bad:

```vue
<template>
  <AudioPlayer
    @playAudio="play"
    @pauseAudio="pause"
    @reloadAudio="reload"
  />
</template>
```

Good:

```vue
<template>
  <AudioPlayer
    @audio:play="play"
    @audio:pause="pause"
    @audio:reload="reload"
  />
</template>
```

## `vue/use-v-on-exact`

Reports an unmodified listener that sits beside the same event with a key
modifier. Without `.exact`, the plain listener also runs when the modifier
matches.

Default severity: `warning`\
Presets: `essential`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <button type="button" @click="handleClick" @click.ctrl="handleCtrlClick">
    Save
  </button>
</template>
```

Good:

```vue
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

## `vue/v-bind-style`

Enforces `v-bind` style. The default, `shorthand`, wants `:attr` rather than
`v-bind:attr`. `longform` is the opposite.

Default severity: `warning`\
Presets: `nuxt`, `opinionated`

Bad:

```vue
<template>
  <div v-bind:class="panelClass"></div>
</template>
```

Good:

```vue
<template>
  <div :class="panelClass"></div>
</template>
```

## `vue/v-on-style`

Enforces `v-on` style. The default, `shorthand`, wants `@event` rather than
`v-on:event`.

Default severity: `warning`\
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <div v-on:click="handleClick"></div>
</template>
```

Good:

```vue
<template>
  <div @click="handleClick"></div>
</template>
```

## `vue/v-slot-style`

Enforces `v-slot` style per position. The default wants `v-slot` on the
component for the default slot, and `#name` on a `<template>` for a named
slot. `#default` on the component, and `v-slot:name` on a template, are the
reported shapes.

Default severity: `warning`\
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <MyComponent #default="props">{{ props.item }}</MyComponent>
  <MyComponent>
    <template v-slot:header>Header</template>
  </MyComponent>
</template>
```

Good:

```vue
<template>
  <MyComponent v-slot="props">{{ props.item }}</MyComponent>
  <MyComponent>
    <template #header>Header</template>
  </MyComponent>
</template>
```

## `vue/warn-custom-directive`

Reports a directive that is not a Vue built-in. The directive still has to
be registered for the app to run. Built-ins such as `v-if`, `v-model`, and
`@click` are not reported.

Default severity: `warning`\
Presets: `nuxt`, `opinionated`

Bad:

```vue
<template>
  <input v-focus />
  <input v-mask="'###-####'" />
  <div v-click-outside="handleClose"></div>
</template>
```

Good:

```vue
<template>
  <div v-if="ready"></div>
  <input v-model="value" />
  <button type="button" @click="onClick">Save</button>
</template>
```
