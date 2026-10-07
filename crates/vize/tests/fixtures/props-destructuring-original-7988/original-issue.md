> **Possibly by design — filed as a question.** both rules are opt-in, so a project can pick one. But the rule has the same base name as eslint-plugin-vue's `vue/define-props-destructuring` with the opposite default and no option, and Vize's own `script/no-with-defaults` recommends exactly what it reports. Whether it should take an option is a design call.

## Area

Linter, `script/define-props-destructuring` and `script/no-with-defaults` (both opt-in)

## Version

`vize` 0.432.0

## Minimal reproduction

`MyBadgeA.vue`

```vue
<script setup lang="ts">
const props = withDefaults(defineProps<{ size?: "sm" | "md" }>(), { size: "md" });
</script>

<template>
  <span :class="props.size">badge</span>
</template>
```

`MyBadgeB.vue`

```vue
<script setup lang="ts">
const { size = "md" } = defineProps<{ size?: "sm" | "md" }>();
</script>

<template>
  <span :class="size">badge</span>
</template>
```

`vize.config.json`

```json
{
  "linter": {
    "preset": "incremental",
    "rules": {
      "script/no-with-defaults": "warn",
      "script/define-props-destructuring": "warn",
      "script/define-props-declaration": "warn"
    }
  }
}
```

```sh
vize lint -f plain --help-level short MyBadgeA.vue MyBadgeB.vue
```

## Actual

```
Patina lint report: 2 warnings in 2 files

MyBadgeA.vue
  MyBadgeA.vue:2:15 warning script/no-with-defaults Prefer destructuring defaults over withDefaults (Vue 3.5+)
    Help:
      Use destructuring with defaults: const { count = 0, name = 'default' } = defineProps<Props>()

MyBadgeB.vue
  MyBadgeB.vue:2:7 warning script/define-props-destructuring Avoid destructuring the return value of defineProps().
    Help:
      Assign the props to a single binding (const props = defineProps(...)) and access props.foo instead of destructuring, which can drop reactivity.
```

With both rules on, each rule's fix is the other rule's violation: `script/no-with-defaults`
recommends the destructuring default that `script/define-props-destructuring` reports. For a
type-based prop with a default, only falling back in code (`props.size ?? "md"`) passes both.

## Questions

1. eslint-plugin-vue's `vue/define-props-destructuring` (since v10.1.0) takes
   `{ destructure: "only-when-assigned" | "always" | "never" }`. Its default,
   `"only-when-assigned"`, requires destructuring when `defineProps` is assigned and reports
   `const props = defineProps()` and `withDefaults(...)`, i.e. the opposite of Vize's rule with the
   same base name; Vize's behaviour corresponds to `"never"` only. Could the Vize rule take the same
   option (via `ruleOptions`), so a project migrating from ESLint keeps its behaviour, and so the two
   Vize rules can be configured consistently?
2. The help text says destructuring "can drop reactivity". Since Vue 3.5, destructured
   `defineProps()` bindings stay reactive (the compiler rewrites them to `__props.x`), which is the
   premise of `script/no-with-defaults`. Maybe the message could say it is a style preference, or
   mention the Vue < 3.5 caveat only.
