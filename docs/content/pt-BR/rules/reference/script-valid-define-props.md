---
title: "script/valid-define-props"
description: "Exigir uso válido de defineProps() (uma única chamada, sem argumentos de tipo e de tempo de execução juntos, sem referências locais)."
---

# `script/valid-define-props`

Exigir uso válido de defineProps() (uma única chamada, sem argumentos de tipo e de tempo de execução juntos, sem referências locais).

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

A mesma chamada de `defineProps` fornece tanto `{ title: string }` como argumento de tipo quanto `{ title: String }` como argumento de tempo de execução, combinação que o compilador não permite.

```vue annotate="remove:2"
<script setup lang="ts">
defineProps<{ title: string }>({ title: String });
</script>
```

## Correto

Remover o objeto de tempo de execução deixa uma única declaração baseada em tipos para `title` em vez de combinar ambas as formas de declaração.

```vue annotate="add:2"
<script setup lang="ts">
defineProps<{ title: string }>();
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/valid-define-props": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_props.rs#L46)
