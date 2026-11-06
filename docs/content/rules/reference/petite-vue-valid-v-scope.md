---
title: "petite-vue/valid-v-scope"
description: "Require v-scope to bind an object literal."
---

# `petite-vue/valid-v-scope`

Require v-scope to bind an object literal.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | None | No | None |

Applies to: HTML documents detected as petite-vue; ordinary Vue SFCs are outside this rule's scope.

## Bad

The four nonempty `v-scope` values are an identifier, a call, arithmetic, and a number; none parses as an object literal.

```html annotate="remove:3,4,5,6"
<!doctype html>
<html><body>
<div v-scope="count"></div>
<div v-scope="foo()"></div>
<div v-scope="a + b"></div>
<div v-scope="123"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

## Good

A valueless `v-scope` uses the root scope. The other values are object literals, including the parenthesized object, which the rule accepts.

```html annotate="add:3,4,5,6"
<!doctype html>
<html><body>
<div v-scope></div>
<div v-scope="{}"></div>
<div v-scope="{ count: 0 }"></div>
<div v-scope="({ count: 0 })"></div>
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
      "petite-vue/valid-v-scope": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [petite-vue rules](../petite-vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_scope.rs#L46)
