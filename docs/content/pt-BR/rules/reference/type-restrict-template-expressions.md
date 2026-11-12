---
title: "type/restrict-template-expressions"
description: "Exigir que as expressões de um template literal sejam compatíveis com string."
---

# `type/restrict-template-expressions`

Exigir que as expressões de um template literal sejam compatíveis com string.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | [Opções das regras](/rules/options.md) |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

O template interpola um array, que não é uma expressão compatível com string.

```vue annotate="remove:2,3"
<script setup lang="ts">
const numbers = [1, 2, 3];
const listText = `${numbers}`;
</script>
```

## Correto

O template interpola label, que é uma string.

```vue annotate="add:2,3"
<script setup lang="ts">
const label = "a";
const listText = `${label}`;
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/restrict-template-expressions": "warn"
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
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/restrict_template_expressions.rs#L5)
