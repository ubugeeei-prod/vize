---
title: "script/no-deprecated-model-definition"
description: "Proibir a opção model do Vue 2, removida no Vue 3."
---

# `script/no-deprecated-model-definition`

Proibir a opção model do Vue 2, removida no Vue 3.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

O componente ainda declara a opção `model` do Vue 2 com `prop` e `event`.

```vue annotate="remove:3,4,5,6"
<script lang="ts">
export default {
  model: {
    prop: 'value',
    event: 'input'
  }
}
</script>
```

## Correto

O componente declara a prop `modelValue` e o evento `update:modelValue`.

```vue annotate="add:3,4"
<script lang="ts">
export default {
  props: ['modelValue'],
  emits: ['update:modelValue']
}
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-model-definition": "error"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_model_definition.rs#L32)
