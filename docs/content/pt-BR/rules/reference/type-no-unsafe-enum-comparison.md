---
title: "type/no-unsafe-enum-comparison"
description: "Proibir comparar um valor de enum com um valor de outro domínio."
---

# `type/no-unsafe-enum-comparison`

Proibir comparar um valor de enum com um valor de outro domínio.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

value é number, então compará-lo com o membro Color.Red cruza domínios de valor.

```vue annotate="remove:6"
<script setup lang="ts">
enum Color {
  Red = 0,
  Green = 1,
}
function isRed(value: number): boolean {
  return value === Color.Red;
}
</script>
```

## Correto

value é Color, então a comparação permanece dentro do enum Color.

```vue annotate="add:6"
<script setup lang="ts">
enum Color {
  Red = 0,
  Green = 1,
}
function isRed(value: Color): boolean {
  return value === Color.Red;
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
      "type/no-unsafe-enum-comparison": "warn"
    },
    "typeAware": true
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_enum_comparison.rs#L5)
