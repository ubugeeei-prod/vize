---
title: "type/no-misused-spread"
description: "Proibir espalhar valores em posições incompatíveis."
---

# `type/no-misused-spread`

Proibir espalhar valores em posições incompatíveis.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | [Opções das regras](/rules/options.md) |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

Espalhar uma string num array coloca um valor numa posição incompatível.

```vue annotate="remove:2,3"
<script setup lang="ts">
const label = "a";
const chars = [...label];
</script>
```

## Correto

Espalhar um array noutro array mantém o tipo dos elementos compatível.

```vue annotate="add:2,3"
<script setup lang="ts">
const numbers = [1, 2, 3];
const chars = [...numbers];
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-misused-spread": "warn"
    },
    "typeAware": true
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Opções das regras](/rules/options.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_misused_spread.rs#L5)
