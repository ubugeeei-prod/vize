---
title: "type/no-unsafe-unary-minus"
description: "Proibir a negação unária em valores que não são number nem bigint."
---

# `type/no-unsafe-unary-minus`

Proibir a negação unária em valores que não são number nem bigint.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

label é string, então o menos unário não nega um number nem um bigint.

```vue annotate="remove:2,3"
<script setup lang="ts">
const label = "a";
const negated = -label;
</script>
```

## Correto

count é um número, então o menos unário nega esse número.

```vue annotate="add:2,3"
<script setup lang="ts">
const count = 1;
const negated = -count;
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unsafe-unary-minus": "warn"
    },
    "typeAware": true
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_unary_minus.rs#L5)
