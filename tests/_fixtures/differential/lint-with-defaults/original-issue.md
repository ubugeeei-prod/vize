## Summary

`script/no-with-defaults` reports the text `withDefaults(` wherever it appears: in comments and in string literals, inside `<script setup>` and in plain `.ts` modules that have no macros at all. Only a real `withDefaults(defineProps…)` call in `<script setup>` should be reported.

Same root cause as #7212 (macro names found in raw text, comments included), which fixed `script/no-export-in-script-setup` but not this rule.

## Reproduction

`vize.config.json`

```json
{ "linter": { "preset": "incremental", "rules": { "script/no-with-defaults": "error" } } }
```

`SizeLabel.vue`

```vue
<script setup lang="ts">
// We no longer use withDefaults() here.
const { size = "m" } = defineProps<{ size?: string }>();
const hint = "withDefaults(";
</script>

<template>
  <p>{{ size }} {{ hint }}</p>
</template>
```

`comment-only.ts`

```ts
/** Replaces `withDefaults(defineProps<Props>(), {})` in older components. */
export const answer = 42;
```

`source-check.ts`

```ts
// Fails when a component still uses the old form.
export const usesOldDefaults = (source: string): boolean => source.includes("withDefaults(");
```

```sh
vize lint -f plain --help-level none SizeLabel.vue comment-only.ts source-check.ts
```

```
SizeLabel.vue:2:21 error script/no-with-defaults Prefer destructuring defaults over withDefaults (Vue 3.5+)
SizeLabel.vue:4:15 error script/no-with-defaults Prefer destructuring defaults over withDefaults (Vue 3.5+)
comment-only.ts:1:15 error script/no-with-defaults Prefer destructuring defaults over withDefaults (Vue 3.5+)
source-check.ts:2:78 error script/no-with-defaults Prefer destructuring defaults over withDefaults (Vue 3.5+)
```

## Expected

No diagnostics. None of the four is a call: two are comments and two are string literals, and the `.ts` files are not `<script setup>` blocks. A real call (`const props = withDefaults(defineProps<…>(), { … })`) should still be reported.

## Why it matters

The rule is useful when migrating to destructured props, and that is exactly when comments and tooling mention the old form: notes about the migration, or a test that reads SFC sources and checks that `withDefaults(` is gone. The only way out is to switch the rule off for those files.

## Environment

- vize 0.432.0
- vue 3.5.42
- node 26.8.1, macOS arm64
