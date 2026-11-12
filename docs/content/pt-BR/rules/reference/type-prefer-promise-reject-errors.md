---
title: "type/prefer-promise-reject-errors"
description: "Exigir que o motivo de rejeição de uma Promise seja semelhante a Error."
---

# `type/prefer-promise-reject-errors`

Exigir que o motivo de rejeição de uma Promise seja semelhante a Error.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | [Opções das regras](/rules/options.md) |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

Promise.reject recebe uma string, que não é um motivo Error.

```vue annotate="remove:2"
<script setup lang="ts">
const rejected = Promise.reject("failure");
</script>
```

## Correto

Promise.reject recebe um Error para o mesmo texto de falha.

```vue annotate="add:2"
<script setup lang="ts">
const rejected = Promise.reject(new Error("failure"));
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/prefer-promise-reject-errors": "warn"
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
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/prefer_promise_reject_errors.rs#L5)
