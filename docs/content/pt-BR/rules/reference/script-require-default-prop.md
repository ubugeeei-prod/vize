---
title: "script/require-default-prop"
description: "Exigir um valor padrão para toda prop opcional que não seja Boolean."
---

# `script/require-default-prop`

Exigir um valor padrão para toda prop opcional que não seja Boolean.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `opinionated`, `nuxt` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

`name` e `age` são props opcionais de tempo de execução que não são Boolean e não têm valores padrão, deixando indefinidos seus valores quando a entrada é omitida.

```vue annotate="remove:4,5,6"
<script lang="ts">
export default {
  props: {
    // optional, non-Boolean, no default
    name: String,
    age: { type: Number },
  }
}
</script>
```

## Correto

`name` recebe `default: ''`. `enabled` usa o valor padrão false implícito de Boolean, e o `id` obrigatório não precisa de um valor alternativo, ilustrando ambas as exceções.

```vue annotate="add:4,5,6"
<script lang="ts">
export default {
  props: {
    name: { type: String, default: '' },
    enabled: Boolean,                 // Boolean defaults to false
    id: { type: Number, required: true },
  }
}
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/require-default-prop": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_default_prop.rs#L58)
