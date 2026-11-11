---
title: "type/require-await"
description: "Proibir funções async que não contêm uma expressão await."
---

# `type/require-await`

Proibir funções async que não contêm uma expressão await.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

noAwait é async, mas o corpo nunca faz await de um valor.

```vue annotate="remove:2,3"
<script setup lang="ts">
async function noAwait(): Promise<number> {
  return 1;
}
</script>
```

## Correto

withAwait faz await de Promise.resolve(1) antes de o devolver.

```vue annotate="add:2,3"
<script setup lang="ts">
async function withAwait(): Promise<number> {
  return await Promise.resolve(1);
}
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/require-await": "warn"
    },
    "typeAware": true
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/require_await.rs#L5)
