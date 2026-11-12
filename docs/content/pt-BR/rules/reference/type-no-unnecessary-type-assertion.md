---
title: "type/no-unnecessary-type-assertion"
description: "Proibir asserções de tipo que não mudam o tipo da expressão."
---

# `type/no-unnecessary-type-assertion`

Proibir asserções de tipo que não mudam o tipo da expressão.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | [Opções das regras](/rules/options.md) |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

known já é number, então afirmar number não muda o seu tipo.

```vue annotate="remove:3"
<script setup lang="ts">
const known: number = 1;
const asserted = known as number;
</script>
```

## Correto

Usar known sem asserção mantém o tipo number anotado.

```vue annotate="add:3"
<script setup lang="ts">
const known: number = 1;
const asserted = known;
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unnecessary-type-assertion": "warn"
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
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unnecessary_type_assertion.rs#L5)
