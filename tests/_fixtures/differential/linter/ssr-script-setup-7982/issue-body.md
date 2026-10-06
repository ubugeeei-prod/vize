## Area

Linter, `ssr/no-browser-globals-in-ssr` (`happy-path`, `nuxt`, `opinionated`)

## Version

`vize` 0.432.0

## Minimal reproduction

`WidthLabel.vue`

```vue
<script setup lang="ts">
const width = window.innerWidth;
const lang = navigator.language;
</script>

<template>
  <p>{{ width }} {{ lang }} {{ document.title }}</p>
</template>
```

`vize.config.json`

```json
{ "linter": { "preset": "incremental", "rules": { "ssr/no-browser-globals-in-ssr": "warn" } } }
```

```sh
vize lint -f plain --help-level none WidthLabel.vue
vize lint --no-config --preset nuxt -f plain --help-level none WidthLabel.vue
```

## Actual

Both commands:

```
Patina lint report: 1 warning in 1 file

WidthLabel.vue
  WidthLabel.vue:7:29 warning ssr/no-browser-globals-in-ssr 'document' is a browser-only global and is not available in SSR
```

Only the template use of `document` is reported. `window.innerWidth` and `navigator.language` at
the top level of `<script setup>` (which runs on the server during SSR) are not.

## Expected

Lines 2 and 3 are reported too. They are the rule's documented "Bad" case,
`docs/content/rules/ssr.md`:

```vue
<script setup lang="ts">
const width = window.innerWidth;
</script>
```

and the doc comment in `no_browser_globals_in_ssr.rs` uses the same example ("This will error in
SSR!"). The "Good" case (`onMounted(() => { width.value = window.innerWidth; })`) should stay
quiet.

## Cause (from reading the source)

`crates/vize_patina/src/rules/ssr/no_browser_globals_in_ssr.rs` only implements
`check_interpolation` and `check_directive` (template expressions); `run_on_template` is a no-op
and there is no script visitor. Related: #7223 (the doctor's `browser-api-ssr` check also missed
`window.*` in setup).
