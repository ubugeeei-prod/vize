---
title: Vue Rule Options
---

# Vue Rule Options

Vue rule option examples cover component name casing, prop mutation depth, SFC block order,
self-closing tags, event names, and attribute casing. The main [Rule Options](./options.md) page lists the complete option table,
unknown-field behavior, and scoped replacement semantics.

## `vue/component-name-in-template-casing`

Use this option when a project wants component tags in templates to be consistently PascalCase or
kebab-case. Native HTML/SVG elements and Vue built-ins are skipped.

```json
{
  "linter": {
    "rules": { "vue/component-name-in-template-casing": "warn" },
    "ruleOptions": {
      "vue/component-name-in-template-casing": { "casing": "kebab-case" }
    }
  }
}
```

Bad with the config above:

```vue
<template>
  <PrimaryButton />
</template>
```

Good:

```vue
<template>
  <primary-button />
</template>
```

## `vue/no-mutating-props`

Use `shallowOnly` when a migration allows nested prop mutation for now but still forbids replacing
the prop binding itself. The default `false` reports both direct and nested mutations.

```json
{
  "linter": {
    "rules": { "vue/no-mutating-props": "error" },
    "ruleOptions": {
      "vue/no-mutating-props": { "shallowOnly": true }
    }
  }
}
```

Bad with the config above:

```vue
<script setup>
const props = defineProps<{ count: number; settings: { dense: boolean } }>();

props.count = 2;
</script>
```

Good with the config above:

```vue
<script setup>
const props = defineProps<{ settings: { dense: boolean } }>();

props.settings.dense = true;
</script>
```

## `vue/sfc-element-order`

Use this option when a project has a fixed SFC block order. A string is one rank; a nested array
means any listed block selector may appear at that rank. Supported built-in selectors are `script`,
`script:not([setup])`, `script[setup]`, `template`, and `style`; other non-empty strings match
custom block names.

```json
{
  "linter": {
    "rules": { "vue/sfc-element-order": "warn" },
    "ruleOptions": {
      "vue/sfc-element-order": {
        "order": ["template", "script:not([setup])", "script[setup]", "i18n", "style"]
      }
    }
  }
}
```

Bad with the config above:

```vue
<style scoped></style>
<template></template>
<script setup></script>
```

Good:

```vue
<template></template>
<script setup></script>
<style scoped></style>
```

## `vue/html-self-closing`

Use this option to choose self-closing style per element family. `always` requires empty elements
to self-close, `never` requires paired start/end tags, and `any` accepts both forms. Omitted nested
fields keep the Vize defaults.

```json
{
  "linter": {
    "rules": { "vue/html-self-closing": "warn" },
    "ruleOptions": {
      "vue/html-self-closing": {
        "html": { "void": "always", "normal": "never", "component": "always" },
        "svg": "always",
        "math": "always"
      }
    }
  }
}
```

Bad with the config above:

```vue
<template>
  <div />
  <PrimaryButton></PrimaryButton>
</template>
```

Good:

```vue
<template>
  <div></div>
  <PrimaryButton />
</template>
```

## `vue/v-on-event-hyphenation`

Use this option for static custom event listener names on components. `always` reports camelCase
listener arguments such as `@saveItem`; `never` reports hyphenated listener arguments. Native HTML
events, object syntax, and dynamic arguments are skipped.

```json
{
  "linter": {
    "rules": { "vue/v-on-event-hyphenation": "warn" },
    "ruleOptions": {
      "vue/v-on-event-hyphenation": "always"
    }
  }
}
```

Bad with the config above:

```vue
<template>
  <PrimaryButton @saveItem="save" />
</template>
```

Good:

```vue
<template>
  <PrimaryButton @save-item="save" />
  <button @saveItem="save" />
</template>
```

## `vue/attribute-hyphenation`

Use this option for static prop names on custom components. `always` reports camelCase-authored
attributes; `never` reports authored attributes that are hyphenated. Native attributes, `aria-*`, `data-*`,
SVG mixed-case attributes, dynamic arguments, and directive shorthand that is parsed as an attribute
are skipped.

```json
{
  "linter": {
    "rules": { "vue/attribute-hyphenation": "warn" },
    "ruleOptions": {
      "vue/attribute-hyphenation": "never"
    }
  }
}
```

Bad with the config above:

```vue
<template>
  <UserCard user-name="Ada" />
</template>
```

Good:

```vue
<template>
  <UserCard userName="Ada" aria-label="Ada Lovelace" />
</template>
```
