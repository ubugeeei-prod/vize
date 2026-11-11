---
title: "type/no-unnecessary-type-arguments"
description: "Proibir argumentos de tipo iguais ao valor padrão."
---

# `type/no-unnecessary-type-arguments`

Proibir argumentos de tipo iguais ao valor padrão.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

O argumento explícito number é igual ao parâmetro de tipo padrão.

```vue annotate="remove:5"
<script setup lang="ts">
function identity<T = number>(value: T): T {
  return value;
}
const fromIdentity = identity<number>(1);
</script>
```

## Correto

Chamar identity(1) deixa o parâmetro de tipo padrão valer.

```vue annotate="add:5"
<script setup lang="ts">
function identity<T = number>(value: T): T {
  return value;
}
const fromIdentity = identity(1);
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unnecessary-type-arguments": "warn"
    },
    "typeAware": true
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unnecessary_type_arguments.rs#L5)
