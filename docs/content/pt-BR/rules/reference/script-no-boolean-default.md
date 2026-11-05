---
title: "script/no-boolean-default"
description: "Proibir um valor padrão em uma prop Boolean."
---

# `script/no-boolean-default`

Proibir um valor padrão em uma prop Boolean.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

Tanto `disabled` quanto `checked` declaram um `default` em uma prop cujo único construtor é `Boolean`; a regra rejeita até mesmo um valor padrão `false` explícito.

```vue annotate="remove:4,5,6"
<script lang="ts">
export default {
  props: {
    // Boolean props already default to false; an explicit default is confusing.
    disabled: { type: Boolean, default: true },
    checked: { type: Boolean, default: false }
  }
}
</script>
```

## Correto

As props exclusivamente Boolean omitem `default`, usando o valor false implícito do Vue. A união `[Boolean, String]` e a prop Number ilustram que esta verificação se limita ao construtor único `Boolean`.

```vue annotate="add:4,5,6,7,8,9,10"
<script lang="ts">
export default {
  props: {
    // No explicit default: defaults to false.
    disabled: { type: Boolean },
    disabled2: Boolean,
    // Union type may legitimately need a default.
    value: { type: [Boolean, String], default: '' },
    // Non-Boolean prop.
    count: { type: Number, default: 0 }
  }
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
        "script/no-boolean-default": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_boolean_default.rs#L54)
