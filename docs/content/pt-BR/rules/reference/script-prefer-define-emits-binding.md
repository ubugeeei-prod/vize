---
title: "script/prefer-define-emits-binding"
description: "Preferir a função capturada de defineEmits ao $emit do template."
---

# `script/prefer-define-emits-binding`

Preferir a função capturada de defineEmits ao $emit do template.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Sim | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

O template chama o auxiliar `$emit` do componente embora `<script setup>` já capture a função tipada retornada por `defineEmits`.

```vue annotate="remove:4"
<script setup lang="ts">
const emit = defineEmits<{ save: [id: number] }>();
</script>
<template><button @click="$emit('save', 1)">Save</button></template>
```

## Correto

O template chama a ligação `emit` capturada, fazendo nomes de eventos e payloads passarem pelo contrato `defineEmits` do componente.

```vue annotate="add:4"
<script setup lang="ts">
const emit = defineEmits<{ save: [id: number] }>();
</script>
<template><button @click="emit('save', 1)">Save</button></template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "script/prefer-define-emits-binding": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/prefer_define_emits_binding.rs#L16)
