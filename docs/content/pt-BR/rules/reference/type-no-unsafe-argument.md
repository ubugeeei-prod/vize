---
title: "type/no-unsafe-argument"
description: "Proibir chamar uma função com um valor do tipo `any`."
---

# `type/no-unsafe-argument`

Proibir chamar uma função com um valor do tipo `any`.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

anyValue é any e é passado a um parâmetro que espera number.

```vue annotate="remove:2,6"
<script setup lang="ts">
declare const anyValue: any;
function takesNumber(value: number): number {
  return value;
}
takesNumber(anyValue);
</script>
```

## Correto

A chamada passa o número 1 ao parâmetro number.

```vue annotate="add:5"
<script setup lang="ts">
function takesNumber(value: number): number {
  return value;
}
takesNumber(1);
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unsafe-argument": "warn"
    },
    "typeAware": true
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_argument.rs#L5)
