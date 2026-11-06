---
title: "script/no-next-tick"
description: "Proibir o uso de nextTick() em componentes orientados a Vapor."
---

# `script/no-next-tick`

Proibir o uso de nextTick() em componentes orientados a Vapor.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | Nenhuma | Não | Nenhuma |

Aplicável a: Verificações de script voltadas ao Vapor; a ativação explícita também aplica a restrição a scripts comuns.

## Incorreto

O componente orientado a Vapor importa e aguarda `nextTick`, introduzindo a dependência do agendamento da atualização do DOM que esta regra de migração rejeita.

```vue annotate="remove:2,3"
<script setup lang="ts" vapor>
import { nextTick } from "vue";
await nextTick();
</script>
```

## Correto

O input é obtido por `useTemplateRef` e recebe foco em `onMounted`. O momento explícito de montagem substitui a dependência de `nextTick` do exemplo.

```vue annotate="add:2,3,4,6"
<script setup lang="ts" vapor>
import { onMounted, useTemplateRef } from "vue";
const input = useTemplateRef<HTMLInputElement>("input");
onMounted(() => { input.value?.focus(); });
</script>
<template><input ref="input"></template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-next-tick": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_next_tick.rs#L40)
