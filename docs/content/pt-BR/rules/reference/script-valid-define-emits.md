---
title: "script/valid-define-emits"
description: "Exigir uso válido de defineEmits() (sem argumentos de tipo e de tempo de execução juntos, sem referências locais, uma única chamada)."
---

# `script/valid-define-emits`

Exigir uso válido de defineEmits() (sem argumentos de tipo e de tempo de execução juntos, sem referências locais, uma única chamada).

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

A mesma chamada de `defineEmits` fornece tanto um argumento de tipo quanto o array de tempo de execução `["save"]`, misturando duas declarações mutuamente exclusivas.

```vue annotate="remove:2"
<script setup lang="ts">
defineEmits<{ save: [] }>(["save"]);
</script>
```

## Correto

Remover o argumento de tempo de execução deixa uma única declaração de evento baseada em tipos para `save`.

```vue annotate="add:2"
<script setup lang="ts">
defineEmits<{ save: [] }>();
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/valid-define-emits": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_emits.rs#L47)
