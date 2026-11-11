---
title: "type/return-await"
description: "Exigir o uso consistente de `return await` conforme o contexto e a configuração."
---

# `type/return-await`

Exigir o uso consistente de `return await` conforme o contexto e a configuração.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

Dentro de try, devolver getNumber() sem await perde a rejeição nesse catch.

```vue annotate="remove:7"
<script setup lang="ts">
function getNumber(): Promise<number> {
  return Promise.resolve(1);
}
async function returnsInTry(): Promise<number> {
  try {
    return getNumber();
  } catch {
    return 0;
  }
}
</script>
```

## Correto

return await getNumber() mantém a rejeição dentro do mesmo try e catch.

```vue annotate="add:7"
<script setup lang="ts">
function getNumber(): Promise<number> {
  return Promise.resolve(1);
}
async function returnsInTry(): Promise<number> {
  try {
    return await getNumber();
  } catch {
    return 0;
  }
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
      "type/return-await": "warn"
    },
    "typeAware": true
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/return_await.rs#L5)
