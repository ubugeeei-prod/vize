---
title: "ecosystem/vue-i18n-no-missing-key"
description: "Report static vue-i18n keys that are absent from local SFC messages."
---

# `ecosystem/vue-i18n-no-missing-key`

Report static vue-i18n keys that are absent from local SFC messages.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `ecosystem` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The template requests auth.missing, but the local English messages declare only auth.login.

```vue annotate="remove:1"
<template>{{ $t("auth.missing") }}</template>

<i18n lang="json">
{ "en": { "auth": { "login": "Log in" } } }
</i18n>
```

## Good

The template requests the auth.login key that exists in the local messages.

```vue annotate="add:1"
<template>{{ $t("auth.login") }}</template>

<i18n lang="json">
{ "en": { "auth": { "login": "Log in" } } }
</i18n>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "ecosystem/vue-i18n-no-missing-key": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Ecosystem and Nuxt rules](../ecosystem.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/i18n_no_missing_key.rs#L17)
