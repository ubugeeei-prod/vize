---
title: "script/prefer-use-template-ref"
description: "Recomendar useTemplateRef em vez de ref para referências de template (Vue 3.5+)."
---

# `script/prefer-use-template-ref`

Recomendar useTemplateRef em vez de ref para referências de template (Vue 3.5+).

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

O ref `input`, que aceita null, está associado ao literal `ref="input"` do template, identificando-o como uma referência de elemento em vez de um dado comum que aceita null.

```vue annotate="remove:2,3"
<script setup lang="ts">
import { ref } from 'vue'
const input = ref<HTMLInputElement | null>(null)
</script>
<template>
  <input ref="input" />
</template>
```

## Correto

`useTemplateRef<HTMLInputElement>('input')` do Vue 3.5+ torna essa referência de template explícita. O `error = ref(null)` sem associação permanece um dado comum e fica intencionalmente fora desta regra.

```vue annotate="add:2,3,4,5,6,10"
<script setup lang="ts">
import { ref, useTemplateRef } from 'vue'
// Paired with the template ref below.
const input = useTemplateRef<HTMLInputElement>('input')
// A nullable data ref the template never binds as a ref.
const error = ref(null)
</script>
<template>
  <input ref="input" />
  <p>{{ error }}</p>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "script/prefer-use-template-ref": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_template_ref.rs#L75)
