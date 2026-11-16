---
title: "type/no-unnecessary-type-parameters"
description: "Proibir parâmetros de tipo que não são usados mais de uma vez."
---

# `type/no-unnecessary-type-parameters`

Proibir parâmetros de tipo que não são usados mais de uma vez.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

T aparece apenas uma vez na assinatura e não expressa uma relação entre tipos.

```vue annotate="remove:2,3"
<script setup lang="ts">
function logValue<T>(value: T): void {
  console.info(value);
}
</script>
```

## Correto

T relaciona o tipo do parâmetro com o tipo de retorno.

```vue annotate="add:2,3"
<script setup lang="ts">
function identity<T>(value: T): T {
  return value;
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
      "type/no-unnecessary-type-parameters": "warn"
    },
    "typeAware": true
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unnecessary_type_parameters.rs#L5)
