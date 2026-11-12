---
title: "type/restrict-plus-operands"
description: "Exigir que os operandos de + sejam explicitamente compatíveis."
---

# `type/restrict-plus-operands`

Exigir que os operandos de + sejam explicitamente compatíveis.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | [Opções das regras](/rules/options.md) |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

label é string e numbers é um array, então os dois operandos de + não são primitivos compatíveis.

```vue annotate="remove:3,4"
<script setup lang="ts">
const label = "a";
const numbers = [1, 2, 3];
const mixedSum = label + numbers;
</script>
```

## Correto

label e extra são ambos string, então os operandos de + coincidem.

```vue annotate="add:3,4"
<script setup lang="ts">
const label = "a";
const extra = "b";
const mixedSum = label + extra;
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/restrict-plus-operands": "warn"
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
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/restrict_plus_operands.rs#L5)
