---
title: "ecosystem/vue-i18n-no-missing-key"
description: "Signaler les clés vue-i18n statiques absentes des messages locaux du SFC."
---

# `ecosystem/vue-i18n-no-missing-key`

Signaler les clés vue-i18n statiques absentes des messages locaux du SFC.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `ecosystem` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le template demande auth.missing, mais les messages anglais locaux ne déclarent que auth.login.

```vue annotate="remove:1"
<template>{{ $t("auth.missing") }}</template>

<i18n lang="json">
{ "en": { "auth": { "login": "Log in" } } }
</i18n>
```

## Bon

Le template demande la clé auth.login qui existe dans les messages locaux.

```vue annotate="add:1"
<template>{{ $t("auth.login") }}</template>

<i18n lang="json">
{ "en": { "auth": { "login": "Log in" } } }
</i18n>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "ecosystem/vue-i18n-no-missing-key": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de l’écosystème et de Nuxt](../ecosystem.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/i18n_no_missing_key.rs#L17)
