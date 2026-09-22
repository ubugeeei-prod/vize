---
title: "Vue Rules: Template Formatting"
---

# Vue Rules: Template Formatting

Attribute conventions, tag style, whitespace, and prop spelling.
See [all Vue rules](./vue.md) for the complete reference and [Vue Rule Options](./options-vue.md)
for configurable settings.

## `vue/attribute-hyphenation`

Enforces attribute naming style on custom components. The default, `always`,
requires hyphenated names. Native HTML attributes are not renamed.

Default severity: `warning`\
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <MyComponent myProp="value" />
  <MyComponent :myProp="value" />
</template>
```

Good:

```vue
<template>
  <MyComponent my-prop="value" />
  <MyComponent :my-prop="value" />
</template>
```

## `vue/attribute-order`

Enforces the Vue style-guide attribute order: `v-for`, then conditionals
(`v-if` / `v-else-if` / `v-else` / `v-show`), then `id`, `ref` / `key`,
`v-model`, other attributes, events, and finally `v-html` / `v-text`.

Default severity: `warning`\
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <div @click="onClick" v-if="show" id="main"></div>
</template>
```

Good:

```vue
<template>
  <div v-if="show" id="main" @click="onClick"></div>
</template>
```

## `vue/html-quotes`

Enforces quote style for HTML attribute values. The default, `double`,
rejects single quotes and unquoted values, including on directives.

Default severity: `warning`\
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <div class='foo'></div>
  <div class=foo></div>
  <div v-if='ready'></div>
</template>
```

Good:

```vue
<template>
  <div class="foo"></div>
  <div v-if="ready"></div>
</template>
```

## `vue/html-self-closing`

Enforces self-closing style. By default, empty components and SVG/MathML
elements, plus HTML void elements, use a slash. Empty ordinary HTML
elements accept either closing style.

Default severity: `warning`\
Presets: `nuxt`, `opinionated`

`linter.ruleOptions["vue/html-self-closing"]` accepts `html.void`,
`html.normal`, `html.component`, `svg`, and `math`. Each value is `"always"`,
`"never"`, or `"any"`.

Bad:

```vue
<template>
  <MyComponent></MyComponent>
  <img>
  <br>
</template>
```

Good:

```vue
<template>
  <MyComponent />
  <div></div>
  <div />
  <img />
  <br />
  <div>content</div>
</template>
```

## `vue/mustache-interpolation-spacing`

Enforces a space inside mustache interpolation. The default, `always`,
reports each delimiter that is missing its space.

Default severity: `warning`\
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <div>{{text}}</div>
  <div>{{ text}}</div>
  <div>{{text }}</div>
</template>
```

Good:

```vue
<template>
  <div>{{ text }}</div>
  <div>{{ foo.bar }}</div>
  <div>{{ foo + bar }}</div>
</template>
```

## `vue/no-boolean-attr-value`

Reports an explicit value on a boolean HTML attribute.

Default severity: `warning`\
Presets: `nuxt`, `opinionated`

Bad:

```vue
<template>
  <input disabled="disabled" />
  <input checked="checked" />
  <button disabled="true">Save</button>
</template>
```

Good:

```vue
<template>
  <input disabled />
  <input checked />
  <button disabled>Save</button>
</template>
```

## `vue/no-inline-style`

Reports inline `style` attributes. A dynamic `:style` binding is allowed,
because layout and theme values often have to be computed.

Default severity: `warning`\
Presets: `nuxt`, `opinionated`

Bad:

```vue
<template>
  <div style="color: red">Text</div>
</template>
```

Good:

```vue
<template>
  <div class="text-red">Text</div>
  <span :class="{ 'text-red': isRed }">Text</span>
  <div :style="{ width: `${ratio}%` }">Text</div>
</template>
```

## `vue/no-multi-spaces`

Reports consecutive spaces in the template.

Default severity: `warning`\
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <div  class="panel"></div>
  <div class="panel"  id="main"></div>
</template>
```

Good:

```vue
<template>
  <div class="panel"></div>
  <div class="panel" id="main"></div>
</template>
```

## `vue/prefer-props-shorthand`

Reports a prop binding whose name matches the value expression. Vue 3.4
writes that as `:foo` instead of `:foo="foo"`. A kebab-case attribute matches
its camelCase value: `:user-name="userName"` is the same case.

Default severity: `warning`\
Presets: `nuxt`, `opinionated`

Bad:

```vue
<template>
  <MyComponent :foo="foo" />
  <MyComponent :user-name="userName" />
</template>
```

Good:

```vue
<template>
  <MyComponent :foo />
  <MyComponent :user-name />
  <MyComponent :foo="bar" />
</template>
```

## `vue/prop-name-casing`

Enforces camelCase for prop names in this component's own `defineProps`
declaration. How a parent writes the attribute is `vue/attribute-hyphenation`,
not this rule.

Default severity: `warning`\
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<script setup lang="ts">
defineProps({ "my-prop": String });
</script>
```

Good:

```vue
<script setup lang="ts">
defineProps({ myProp: String });
</script>
```

## `vue/valid-attribute-name`

Reports invalid characters in attribute names, such as quotes, whitespace,
or control characters. A leading digit is allowed.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <div my"attr="value"></div>
</template>
```

Good:

```vue
<template>
  <div my-attr="value"></div>
  <div data-value="value"></div>
</template>
```
