---
title: "type/no-misused-promises"
description: "Proibir Promises em lugares que não foram feitos para tratá-las."
---

# `type/no-misused-promises`

Proibir Promises em lugares que não foram feitos para tratá-las.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

forEach espera um callback void, mas o callback async devolve uma Promise.

```vue annotate="remove:3"
<script setup lang="ts">
const numbers = [1, 2, 3];
numbers.forEach(async (n) => {
  console.log(n);
});
</script>
```

## Correto

O callback não é async, então devolve void como forEach espera.

```vue annotate="add:3"
<script setup lang="ts">
const numbers = [1, 2, 3];
numbers.forEach((n) => {
  console.log(n);
});
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-misused-promises": "warn"
    },
    "typeAware": true
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_misused_promises.rs#L5)
