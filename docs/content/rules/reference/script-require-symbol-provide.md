---
title: "script/require-symbol-provide"
description: "Recommend using Symbol as injection key for provide/inject."
---

# `script/require-symbol-provide`

Recommend using Symbol as injection key for provide/inject.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

`provide` and `inject` use literal string keys such as `'user'` and `'theme'`, which can collide with another provider using the same spelling.

```vue annotate="remove:1,2,3,4,6,7"
<script setup lang="ts">
// String keys can collide
provide('user', user)
const user = inject('user')

// Magic strings are error-prone
provide('theme', { dark: true })
</script>
```

## Good

The shared `UserKey` is created with `Symbol` and annotated as `InjectionKey<User>`; both calls pass that key instead of a literal string.

```vue annotate="add:1,2,3,5,6,7,8,9"
<script lang="ts">
// Define injection key with Symbol
export const UserKey: InjectionKey<User> = Symbol('user')

// Provide with Symbol
provide(UserKey, user)

// Inject with Symbol
const user = inject(UserKey)
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "script/require-symbol-provide": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_symbol_provide.rs#L39)
