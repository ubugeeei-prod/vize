---
title: "vapor/no-vue-lifecycle-events"
description: "Proibir eventos de ciclo de vida @vue:xxx por elemento (não suportados em Vapor)."
---

# `vapor/no-vue-lifecycle-events`

Proibir eventos de ciclo de vida @vue:xxx por elemento (não suportados em Vapor).

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O input usa o evento de ciclo de vida de template @vue:mounted.

```vue annotate="remove:2"
<template>
  <input @vue:mounted="focusInput" />
</template>
```

## Correto

onMounted acessa a referência nomeada de template e coloca o foco no input por meio do hook de ciclo de vida de script suportado.

```vue annotate="add:1,2,3,4,5,6,7,8,10"
<script setup lang="ts" vapor>
const input = useTemplateRef<HTMLInputElement>("input");

onMounted(() => {
  input.value?.focus();
});
</script>

<template>
  <input ref="input" />
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vapor/no-vue-lifecycle-events": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de Vapor](../vapor.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vapor/no_vue_lifecycle_events.rs#L34)
