---
title: "script/custom-event-name-casing"
description: "Exigir a capitalização configurada nos nomes de eventos personalizados emitidos."
---

# `script/custom-event-name-casing`

Exigir a capitalização configurada nos nomes de eventos personalizados emitidos.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `opinionated`, `nuxt` | Não | [Opções das regras](/rules/options.md) |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

A string emitida `my-event` contém um hífen e viola a política padrão de nomes de eventos em camelCase.

```vue annotate="remove:2,3"
<script setup lang="ts">
const emit = defineEmits(['my-event'])
emit('my-event')         // kebab-case → report
</script>
```

## Correto

Tanto a declaração quanto a chamada usam `myEvent`, atendendo ao padrão global camelCase. O preset `opinionated` seleciona kebab-case, e as opções explícitas da regra também permitem nomes totalmente minúsculos como `transitionstart`.

```vue annotate="add:2,3"
<script setup lang="ts">
const emit = defineEmits(['myEvent'])
emit('myEvent')
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
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/custom_event_name_casing.rs#L63)
