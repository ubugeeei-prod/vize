---
title: "script/custom-event-name-casing"
description: "Aplicar o formato configurado aos nomes de eventos personalizados emitidos."
---

# `script/custom-event-name-casing`

Aplicar o formato configurado aos nomes de eventos personalizados emitidos.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `opinionated`, `nuxt` | Não | [Opções das regras](/rules/options.md) |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

A string emitida `transitionStart` contém uma letra maiúscula e viola a política padrão de minúsculas sem separadores.

```vue annotate="remove:2,3"
<script setup lang="ts">
const emit = defineEmits(['transitionStart'])
emit('transitionStart')         // camelCase → report
</script>
```

## Correto

Tanto a declaração quanto a chamada usam `transitionstart`, preservando a correspondência e seguindo a convenção de plataforma em minúsculas. O projeto pode configurar camelCase ou kebab-case explicitamente.

```vue annotate="add:2,3"
<script setup lang="ts">
const emit = defineEmits(['transitionstart'])
emit('transitionstart')
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/custom-event-name-casing": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Opções das regras](/rules/options.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/custom_event_name_casing.rs#L62)
