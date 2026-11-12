---
title: "type/only-throw-error"
description: "Exigir que os valores lançados sejam semelhantes a Error."
---

# `type/only-throw-error`

Exigir que os valores lançados sejam semelhantes a Error.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | [Opções das regras](/rules/options.md) |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

O valor lançado é uma string, que não é um Error.

```vue annotate="remove:3"
<script setup lang="ts">
function fail(): never {
  throw "failure";
}
</script>
```

## Correto

new Error cria um Error para o mesmo texto de falha.

```vue annotate="add:3"
<script setup lang="ts">
function fail(): never {
  throw new Error("failure");
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
      "type/only-throw-error": "warn"
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
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/only_throw_error.rs#L5)
