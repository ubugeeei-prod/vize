---
title: "type/no-unnecessary-type-parameters"
description: "Proibir parâmetros de tipo usados uma única vez na assinatura."
---

# `type/no-unnecessary-type-parameters`

Proibir parâmetros de tipo usados uma única vez na assinatura.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

T aparece uma única vez na assinatura de logValue, então não relaciona nada.

```vue annotate="remove:2"
<script setup lang="ts">
function logValue<T>(value: T): void {
  console.info(value);
}
logValue(1);
</script>
```

## Correto

Tipar value como unknown aceita os mesmos argumentos sem parâmetro de tipo.

```vue annotate="add:2"
<script setup lang="ts">
function logValue(value: unknown): void {
  console.info(value);
}
logValue(1);
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
