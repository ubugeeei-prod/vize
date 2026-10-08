## Summary

When an SFC has a `<script>` or `<script setup>` block, `vize build` does not report template compilation errors. It prints `✓ 1 file compiled`, exits 0, and writes a component **without a render function** (the template is silently dropped). The error only shows up in the `errors` array of `-f json`.

The same template errors in a template-only SFC are reported correctly (`✗ 1 file(s) failed`, exit 1). So a typo in a template can ship as a blank component without any signal from the CLI. This also applies to Vapor-only errors such as `v-memo with dependencies is not supported in Vapor yet`.

## Environment

- `vize` 0.432.0 (npm)
- macOS 26.4.1 arm64, Node v26.10.0, npm 11.19.1

## Reproduction

```sh
mkdir repro && cd repro
printf '<script setup>\nconst a = 1\n</script>\n\n<template>\n  <div>{{ a + }}</div>\n</template>\n' > WithScript.vue
printf '<template>\n  <div>{{ a + }}</div>\n</template>\n' > TemplateOnly.vue
printf '<script setup>\nconst a = 1\n</script>\n\n<template>\n  <p v-if>{{ a }}</p>\n</template>\n' > VIf.vue

npx vize@0.432.0 build --no-config -o out WithScript.vue; echo "exit=$?"
npx vize@0.432.0 build --no-config -o out TemplateOnly.vue; echo "exit=$?"
npx vize@0.432.0 build --no-config -o out VIf.vue; echo "exit=$?"
cat out/WithScript.js
npx vize@0.432.0 build --no-config -f json -o json WithScript.vue && cat json/WithScript.json
```

## Actual

- `WithScript.vue` and `VIf.vue`: `✓ 1 file compiled`, `exit=0`. `out/WithScript.js` is only the setup function (`return { a }`): no `render`, no template code.
- `TemplateOnly.vue`: `✗ 1 error(s) occurred … InvalidExpression … ✗ 1 file(s) failed`, `exit=1`.
- `-f json` for `WithScript.vue` has `"errors": ["Template compilation errors: [CompilerError { code: InvalidExpression, … }]"]`, but the run still reports success.

Same with `--continue-on-error`, and with `<script>` (Options API) instead of `<script setup>`.

## Expected

Template errors fail the build the same way whether or not the SFC has a script block: report the error, count the file as failed, exit non-zero, and do not write a render-less component (as `@vue/compiler-sfc` / `@vitejs/plugin-vue` do: the error is thrown).
