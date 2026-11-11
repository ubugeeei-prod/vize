---
title: "type/no-unsafe-return"
description: "Proibir devolver valores any a partir de funções."
---

# `type/no-unsafe-return`

Proibir devolver valores any a partir de funções.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

A função promete number, mas devolve anyValue, que é any.

```vue annotate="remove:2,4"
<script setup lang="ts">
declare const anyValue: any;
function returnsAny(): number {
  return anyValue;
}
</script>
```

## Correto

A função devolve o número 1, que corresponde ao tipo de retorno anotado.

```vue annotate="add:3"
<script setup lang="ts">
function returnsAny(): number {
  return 1;
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
      "type/no-unsafe-return": "warn"
    },
    "typeAware": true
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_return.rs#L5)
