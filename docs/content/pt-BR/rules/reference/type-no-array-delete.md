---
title: "type/no-array-delete"
description: "Proibir delete de elementos em valores semelhantes a array."
---

# `type/no-array-delete`

Proibir delete de elementos em valores semelhantes a array.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

delete numbers[0] remove um elemento e deixa um buraco em vez de reindexar o array.

```vue annotate="remove:3"
<script setup lang="ts">
const numbers = [1, 2, 3];
delete numbers[0];
</script>
```

## Correto

splice(0, 1) remove o primeiro elemento e mantém o array denso.

```vue annotate="add:3"
<script setup lang="ts">
const numbers = [1, 2, 3];
numbers.splice(0, 1);
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-array-delete": "warn"
    },
    "typeAware": true
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_array_delete.rs#L5)
