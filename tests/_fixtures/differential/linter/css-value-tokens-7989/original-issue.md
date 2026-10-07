## Area

Linter: rules that match the raw source text instead of the AST / parsed stylesheet

## Version

`vize` 0.432.0

Same class of problem as #7210 (`css/prefer-slotted`, fixed), #7962 (`script/no-with-defaults`) and
the `script/require-function-return-type` / `css/no-utility-classes` reports. This one covers four
more rules found while running every rule over a real code base.

## Minimal reproduction

`MyNotes.vue`

```vue
<script setup lang="ts">
import { ref } from "vue";

// We moved away from reactive({ ... }) and from Math.random().toString(36) ids.
const hint = ref("reactive({ a: 1 }) is not used here");
</script>

<template>
  <p class="my-notes">{{ hint }}</p>
</template>

<style scoped>
/* The old rule used v-bind(color); it is gone. */
.my-notes::after {
  content: "v-bind(size) !important";
}
</style>
```

`vize.config.json`

```json
{
  "linter": {
    "preset": "incremental",
    "rules": {
      "script/prefer-ref-over-reactive": "warn",
      "script/prefer-use-id": "warn",
      "css/no-v-bind-performance": "warn",
      "css/no-important": "warn"
    }
  }
}
```

```sh
vize lint -f plain --help-level none MyNotes.vue
```

## Actual

```
Patina lint report: 6 warnings in 1 file

MyNotes.vue
  MyNotes.vue:4:23 warning script/prefer-ref-over-reactive Consider using ref() instead of reactive() for simpler state management
  MyNotes.vue:4:50 warning script/prefer-use-id Consider using useId() for generating IDs (Vue 3.5+)
  MyNotes.vue:5:19 warning script/prefer-ref-over-reactive Consider using ref() instead of reactive() for simpler state management
  MyNotes.vue:13:22 warning css/no-v-bind-performance v-bind() installs runtime CSS variable updates
  MyNotes.vue:15:13 warning css/no-v-bind-performance v-bind() installs runtime CSS variable updates
  MyNotes.vue:15:26 warning css/no-important Avoid using !important as it makes styles harder to override
```

| position | rule | what it is |
| --- | --- | --- |
| 4:23 | `script/prefer-ref-over-reactive` | `reactive(` in a `//` comment |
| 4:50 | `script/prefer-use-id` | `Math.random().toString(36)` in a `//` comment |
| 5:19 | `script/prefer-ref-over-reactive` | `reactive(` in a string literal |
| 13:22 | `css/no-v-bind-performance` | `v-bind(` in a CSS comment |
| 15:13 | `css/no-v-bind-performance` | `v-bind(` in a CSS string (`content`) |
| 15:26 | `css/no-important` | `!important` inside a CSS string (`content`) |

## Expected

No diagnostics: the file has no `reactive()` call, no `Math.random()` id, no CSS `v-bind()` and no
`!important` declaration. (`css/no-important` already skips CSS comments; only the string case is
wrong.)

## Cause (from reading the source)

`prefer_ref_over_reactive.rs`, `prefer_use_id.rs`, `no_v_bind_performance.rs` and
`no_important.rs` in `crates/vize_patina/src/rules/{script,css}/` search the block text with
`memmem` and report each hit. Using the script AST (`CallExpression` with callee `reactive`, a
`Math.random()` member chain) and the parsed stylesheet (declaration values / `!important` flags,
which Lightning CSS already exposes) would avoid comments and strings.
