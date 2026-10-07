## Summary

`vize lint --fix` applies one round of edits. When two fixes overlap on the same range, only one is applied, and the next run applies the other. With the `opinionated` preset, `v-bind:title="title"` gets `vue/v-bind-style` (→ `:title="title"`) and `vue/prefer-props-shorthand` (→ `:title`). After `--fix`, the file still has a fixable warning, so a CI step like `vize lint --fix && git diff --exit-code` or a pre-commit hook gives a different result each time it runs.

ESLint re-lints and re-applies fixes (up to 10 passes, `MAX_AUTOFIX_PASSES`) until no fixable problems remain. One `--fix` invocation then produces the final output.

## Environment

- `vize` 0.432.0 (npm)
- macOS 26.4.1 arm64, Node v26.10.0, npm 11.19.1

## Reproduction

```sh
mkdir -p repro-fix-passes && cd repro-fix-passes
printf '<script setup lang="ts">\nconst title = "x";\n</script>\n\n<template>\n  <div v-bind:title="title" />\n</template>\n' > TitleBox.vue
npx vize@0.432.0 lint --no-config --preset opinionated -f plain --help-level none TitleBox.vue | grep warning
npx vize@0.432.0 lint --no-config --preset opinionated --fix -q TitleBox.vue > /dev/null
echo "after 1st --fix: $(sed -n 6p TitleBox.vue)"
npx vize@0.432.0 lint --no-config --preset opinionated -f plain --help-level none TitleBox.vue | grep warning
npx vize@0.432.0 lint --no-config --preset opinionated --fix -q TitleBox.vue > /dev/null
echo "after 2nd --fix: $(sed -n 6p TitleBox.vue)"
```

## Actual

```text
  TitleBox.vue:6:8 warning vue/prefer-props-shorthand Use shorthand syntax for same-name prop binding
  TitleBox.vue:6:8 warning vue/v-bind-style Prefer shorthand `:` over `v-bind:`
after 1st --fix:   <div :title="title" />
  TitleBox.vue:6:8 warning vue/prefer-props-shorthand Use shorthand syntax for same-name prop binding
after 2nd --fix:   <div :title />
```

## Expected

One `--fix` run produces `<div :title />`, with no fixable diagnostics left. `--fix` should re-lint the fixed source and apply the remaining fixes, with a pass limit like ESLint's.
