---
title: "type/await-thenable"
description: "Proibir await de valores que não são thenable."
---

# `type/await-thenable`

Proibir await de valores que não são thenable.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | [Opções das regras](/rules/options.md) |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

plainNumber devolve um número, portanto o await espera um valor que não é thenable.

```vue annotate="remove:2,3,4,6"
<script setup lang="ts">
function plainNumber(): number {
  return 1;
}
async function run(): Promise<number> {
  return await plainNumber();
}
</script>
```

## Correto

Promise.resolve(1) é thenable, portanto o await espera uma Promise real.

```vue annotate="add:3"
<script setup lang="ts">
async function run(): Promise<number> {
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
      "type/await-thenable": "warn"
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
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/await_thenable.rs#L5)
