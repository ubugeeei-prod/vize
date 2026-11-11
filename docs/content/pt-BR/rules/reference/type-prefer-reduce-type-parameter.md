---
title: "type/prefer-reduce-type-parameter"
description: "Preferir o parâmetro de tipo de Array#reduce a uma asserção do valor inicial."
---

# `type/prefer-reduce-type-parameter`

Preferir o parâmetro de tipo de Array#reduce a uma asserção do valor inicial.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

O valor inicial é afirmado como number[] em vez de usar o parâmetro de tipo de reduce.

```vue annotate="remove:3"
<script setup lang="ts">
const numbers = [1, 2, 3];
const collected = numbers.reduce((all, n) => [...all, n], [] as number[]);
</script>
```

## Correto

reduce<number[]> declara o tipo do acumulador sem afirmar o array inicial.

```vue annotate="add:3"
<script setup lang="ts">
const numbers = [1, 2, 3];
const collected = numbers.reduce<number[]>((all, n) => [...all, n], []);
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/prefer-reduce-type-parameter": "warn"
    },
    "typeAware": true
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/prefer_reduce_type_parameter.rs#L5)
