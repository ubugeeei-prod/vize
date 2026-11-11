---
title: "type/no-confusing-void-expression"
description: "Proibir expressões do tipo void em posições de valor confusas."
---

# `type/no-confusing-void-expression`

Proibir expressões do tipo void em posições de valor confusas.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

logVoid() devolve void, mas o inicializador trata essa chamada como um valor.

```vue annotate="remove:3"
<script setup lang="ts">
function logVoid(): void {}
const value = logVoid();
</script>
```

## Correto

Chamar logVoid() como instrução usa o resultado void apenas pelo seu efeito.

```vue annotate="add:3"
<script setup lang="ts">
function logVoid(): void {}
logVoid();
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-confusing-void-expression": "warn"
    },
    "typeAware": true
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_confusing_void_expression.rs#L5)
