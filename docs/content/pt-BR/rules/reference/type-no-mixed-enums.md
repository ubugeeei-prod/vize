---
title: "type/no-mixed-enums"
description: "Proibir a mistura de membros string e numéricos num enum."
---

# `type/no-mixed-enums`

Proibir a mistura de membros string e numéricos num enum.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

O enum mistura um membro numérico com um membro string.

```vue annotate="remove:4"
<script setup lang="ts">
enum Mixed {
  A = 0,
  B = "b",
}
</script>
```

## Correto

Os dois membros são numéricos, então o enum tem um único domínio de valores.

```vue annotate="add:4"
<script setup lang="ts">
enum Mixed {
  A = 0,
  B = 1,
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
      "type/no-mixed-enums": "warn"
    },
    "typeAware": true
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_mixed_enums.rs#L5)
