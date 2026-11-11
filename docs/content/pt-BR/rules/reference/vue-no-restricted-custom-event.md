---
title: "vue/no-restricted-custom-event"
description: "Proíbe o evento personalizado configurado."
---

# `vue/no-restricted-custom-event`

Proíbe o evento personalizado configurado.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O template emite o evento proibido legacyEvent.

```vue annotate="remove:2"
<template>
  <button type="button" @click="$emit('legacyEvent')">x</button>
</template>
```

## Correto

O template emite o evento save.

```vue annotate="add:2"
<template>
  <button type="button" @click="$emit('save')">x</button>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-custom-event": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-custom-event": [
        "legacyEvent"
      ]
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/restricted_custom_event.rs#L19)
