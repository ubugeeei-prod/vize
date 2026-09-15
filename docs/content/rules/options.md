---
title: Rule Options
---

# Rule Options

`linter.ruleOptions` holds typed project-local settings for rules that accept options. Unknown
option fields are rejected, and later matching config entries replace the full option object for the
same rule. Set severity separately under `linter.rules`; options only describe how an enabled rule
should behave unless noted below.

```json
{
  "linter": {
    "rules": {
      "script/no-restricted-globals": "error",
      "vue/html-self-closing": "warn",
      "musea/prefer-design-tokens": "warn"
    },
    "ruleOptions": {
      "script/no-restricted-globals": {
        "globals": [
          { "name": "process", "message": "Read env via a typed helper." },
          { "name": "alert" }
        ]
      },
      "vue/html-self-closing": {
        "html": { "void": "always", "normal": "never", "component": "always" },
        "svg": "always",
        "math": "always"
      },
      "musea/prefer-design-tokens": {
        "tokens": [
          { "path": "color.primary", "value": "#3b82f6", "tier": "semantic" }
        ]
      }
    }
  }
}
```

| Rule | Option shape | Defaults and behavior |
| --- | --- | --- |
| `script/no-restricted-globals` | `{ globals?: Array<{ name: string; message?: string }> }` | Without options, the built-in deny list is `process`, `localStorage`, and `sessionStorage`. A non-empty `globals` list replaces that built-in list. |
| `script/no-restricted-members` | `{ members?: Array<{ object: string; property: string; message?: string }> }` | Off unless `members` is configured and the rule is enabled. A missing `message` uses the generic diagnostic help. |
| `vue/component-name-in-template-casing` | `{ casing?: "PascalCase" \| "kebab-case" }` | Defaults to `PascalCase`. |
| `script/custom-event-name-casing` | `{ casing?: "camelCase" \| "kebab-case" }` | Defaults to `camelCase`. |
| `vue/no-mutating-props` | `{ shallowOnly?: boolean }` | Defaults to `false`; `true` allows nested mutation while still disallowing direct prop reassignment. |
| `vue/sfc-element-order` | `{ order?: Array<string \| string[]> }` | Defaults to `[["script", "template"], "style"]`. A nested array means any of those block selectors may appear at that rank. |
| `vue/html-self-closing` | `{ html?: { void?: Style; normal?: Style; component?: Style }; svg?: Style; math?: Style }`, where `Style` is `"always"`, `"never"`, or `"any"` | Defaults are `html.void: "always"`, `html.normal: "any"`, `html.component: "always"`, `svg: "always"`, and `math: "always"`. |
| `vue/v-on-event-hyphenation` | `"always" \| "never"` | Configures whether event listener names should be hyphenated. |
| `vue/attribute-hyphenation` | `"always" \| "never"` | Configures whether component prop attributes should be hyphenated in templates. |
| `musea/prefer-design-tokens` | `{ tokens?: Array<{ path: string; value: string; tier?: string }> }` | Off unless token data is configured and the rule is enabled or implicitly selected by a non-empty token list. `tier` defaults to `primitive`. |

## Scoped Entries

When `entries` match a file, their `linter.ruleOptions` overlay the root options for that file.
Each configured rule option object replaces the same root object; arrays such as `globals`,
`members`, `order`, and `tokens` are not concatenated across entries.

## `script/no-restricted-globals`

Use this rule when runtime globals must go through project-owned wrappers. Without options, the
rule reports `process`, `localStorage`, and `sessionStorage`. A non-empty `globals` list replaces
that built-in list; an empty list falls back to the built-ins so a config typo cannot silently
disable the rule.

```json
{
  "linter": {
    "rules": { "script/no-restricted-globals": "error" },
    "ruleOptions": {
      "script/no-restricted-globals": {
        "globals": [
          { "name": "process", "message": "Read env through useRuntimeConfig()." },
          { "name": "localStorage" }
        ]
      }
    }
  }
}
```

Bad:

```ts
const flag = process.env.FEATURE_FLAG;
const token = localStorage.getItem("auth.token");
```

Good:

```ts
const flag = useRuntimeConfig().featureFlag;
const token = authStorage.read("auth.token");
```

## `script/no-restricted-members`

Use this rule for project-local member access bans, such as moving browser APIs behind SSR-safe
helpers. The rule has no built-in deny list; it only reports when `members` is non-empty and the
rule is enabled. Each entry matches a bare identifier receiver plus a static property name.

```json
{
  "linter": {
    "rules": { "script/no-restricted-members": "error" },
    "ruleOptions": {
      "script/no-restricted-members": {
        "members": [
          { "object": "window", "property": "localStorage", "message": "Use authStorage." },
          { "object": "globalThis", "property": "process" }
        ]
      }
    }
  }
}
```

Bad:

```ts
const token = window.localStorage.getItem("auth.token");
const env = globalThis.process.env;
```

Good:

```ts
const token = authStorage.read("auth.token");
const env = readServerEnv();
```

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

## `script/custom-event-name-casing`

Use this option when emitted custom events should follow one casing convention across both script
and template usage. The default is `camelCase`; set `kebab-case` for projects that author emitted
event names in their template-facing form.

```json
{
  "linter": {
    "rules": { "script/custom-event-name-casing": "error" },
    "ruleOptions": {
      "script/custom-event-name-casing": { "casing": "kebab-case" }
    }
  }
}
```

Bad with the config above:

```ts
const emit = defineEmits(["saveItem"]);
emit("saveItem");
```

Good:

```ts
const emit = defineEmits(["save-item"]);
emit("save-item");
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

Use this option for static prop names on custom components. `always` reports camelCase authored
attributes; `never` reports hyphenated authored attributes. Native attributes, `aria-*`, `data-*`,
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

## `musea/prefer-design-tokens`

Use this option to connect Musea/style linting to the project's design-token inventory. The rule is
off unless token data is configured and the rule is enabled or selected by a non-empty token list.
Each token maps a hardcoded CSS value to a CSS custom property derived from `path`.

```json
{
  "linter": {
    "rules": { "musea/prefer-design-tokens": "warn" },
    "ruleOptions": {
      "musea/prefer-design-tokens": {
        "tokens": [
          { "path": "color.primary", "value": "#3b82f6" },
          { "path": "color.danger", "value": "#ef4444", "tier": "semantic" }
        ]
      }
    }
  }
}
```

Bad with the config above:

```vue
<style scoped>
.button {
  color: #3b82f6;
}
</style>
```

Good:

```vue
<style scoped>
.button {
  color: var(--color-primary);
}
</style>
```
