---
title: "type/no-unnecessary-condition"
description: "Proibir condições que são sempre verdadeiras, sempre falsas, never ou sempre nullish."
---

# `type/no-unnecessary-condition`

Proibir condições que são sempre verdadeiras, sempre falsas, never ou sempre nullish.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

value é string, então nunca é nullish e o fallback de ?? nunca executa.

```vue annotate="remove:2"
<script setup lang="ts">
function orDefault(value: string): string {
  return value ?? "x";
}
</script>
```

## Correto

value pode ser undefined, então ?? fornece uma string quando o valor está ausente.

```vue annotate="add:2"
<script setup lang="ts">
function orDefault(value: string | undefined): string {
  return value ?? "x";
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
      "type/no-unnecessary-condition": "warn"
    },
    "typeAware": true
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unnecessary_condition.rs#L5)
