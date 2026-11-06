---
title: "petite-vue/valid-v-effect"
description: "Require v-effect to have a non-empty expression."
---

# `petite-vue/valid-v-effect`

Require v-effect to have a non-empty expression.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | None | No | None |

Applies to: HTML documents detected as petite-vue; ordinary Vue SFCs are outside this rule's scope.

## Bad

Each `v-effect` has no executable expression: its value is missing, empty, or only whitespace.

```html annotate="remove:3,4,5"
<!doctype html>
<html><body>
<div v-effect></div>
<div v-effect=""></div>
<div v-effect="   "></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

## Good

Both `v-effect` values contain an expression: one updates `el.textContent`, and the other increments `count`. This rule checks for a nonempty expression, not the effect’s business logic.

```html annotate="add:3,4"
<!doctype html>
<html><body>
<div v-effect="el.textContent = count"></div>
<div v-effect="count++"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "petite-vue/valid-v-effect": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [petite-vue rules](../petite-vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_effect.rs#L35)
