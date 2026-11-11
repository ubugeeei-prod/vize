---
title: "type/no-unnecessary-type-conversion"
description: "Proibir conversões que não mudam o tipo nem o valor da expressão."
---

# `type/no-unnecessary-type-conversion`

Proibir conversões que não mudam o tipo nem o valor da expressão.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

label já é string, então String(label) não muda o seu tipo nem o seu valor.

```vue annotate="remove:2,3"
<script setup lang="ts">
const label = "a";
const same = String(label);
</script>
```

## Correto

count é um número, então String(count) converte-o numa string.

```vue annotate="add:2,3"
<script setup lang="ts">
const count = 1;
const same = String(count);
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unnecessary-type-conversion": "warn"
    },
    "typeAware": true
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unnecessary_type_conversion.rs#L5)
