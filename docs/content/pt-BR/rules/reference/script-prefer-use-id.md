---
title: "script/prefer-use-id"
description: "Recomendar o uso de useId() para gerar IDs únicos (Vue 3.5+)."
---

# `script/prefer-use-id`

Recomendar o uso de useId() para gerar IDs únicos (Vue 3.5+).

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

`id` contém `Math.random()`, então o identificador gerado para input/label pode diferir entre a renderização no servidor e no cliente. A variável com nome de ID é o contexto de geração reconhecido pela regra.

```vue annotate="remove:2"
<script setup lang="ts">
const id = `input-${Math.random()}`;
</script>
<template><label :for="id">Name</label><input :id="id" /></template>
```

## Correto

`useId()` do Vue 3.5+ gera o identificador, e tanto `:for` quanto `:id` continuam lendo a mesma variável em vez de gerar valores aleatórios de forma independente.

```vue annotate="add:2,3"
<script setup lang="ts">
import { useId } from "vue";
const id = useId();
</script>
<template><label :for="id">Name</label><input :id="id" /></template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "script/prefer-use-id": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_id.rs#L45)
