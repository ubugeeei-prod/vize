---
title: "script/no-template-ref-outside-instance"
description: "Proibir useTemplateRef() onde nenhuma instância de componente o executa."
---

# `script/no-template-ref-outside-instance`

Proibir useTemplateRef() onde nenhuma instância de componente o executa.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

`focus()` só roda como handler de clique do botão, e o callback de `setTimeout` roda depois que o setup terminou. Nenhum dos dois tem uma instância de componente atual, então as refs que criam nunca são preenchidas.

```vue annotate="remove:2,4,6"
<script setup lang="ts">
import { useTemplateRef } from 'vue'
function focus() {
  useTemplateRef<HTMLInputElement>('input').value?.focus()
}
setTimeout(() => useTemplateRef('input'))
</script>
<template>
  <input ref="input" />
  <button @click="focus">Focus</button>
</template>
```

## Correto

`useTemplateRef('input')` roda uma única vez no setup. O handler de clique e o callback de `onMounted` apenas leem `input.value`.

```vue annotate="add:2,3,5,7"
<script setup lang="ts">
import { onMounted, useTemplateRef } from 'vue'
const input = useTemplateRef<HTMLInputElement>('input')
function focus() {
  input.value?.focus()
}
onMounted(() => focus())
</script>
<template>
  <input ref="input" />
  <button @click="focus">Focus</button>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-template-ref-outside-instance": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_template_ref_outside_instance.rs#L78)
