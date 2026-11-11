---
title: "type/no-duplicate-type-constituents"
description: "Proibir constituintes duplicados em uniões ou interseções."
---

# `type/no-duplicate-type-constituents`

Proibir constituintes duplicados em uniões ou interseções.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

string aparece duas vezes na mesma união, então o segundo constituinte não acrescenta nada.

```vue annotate="remove:2"
<script setup lang="ts">
type Dup = string | string;
</script>
```

## Correto

O alias mantém um único constituinte string.

```vue annotate="add:2"
<script setup lang="ts">
type Dup = string;
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-duplicate-type-constituents": "warn"
    },
    "typeAware": true
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_duplicate_type_constituents.rs#L5)
