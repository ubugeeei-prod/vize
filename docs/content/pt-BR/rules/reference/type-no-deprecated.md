---
title: "type/no-deprecated"
description: "Proibir o uso de declarações obsoletas."
---

# `type/no-deprecated`

Proibir o uso de declarações obsoletas.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

legacy está marcado com @deprecated, e a chamada ainda usa essa declaração.

```vue annotate="remove:2,3,6"
<script setup lang="ts">
/** @deprecated use current() */
function legacy(): number {
  return 1;
}
const legacyValue = legacy();
</script>
```

## Correto

current não tem marca de obsolescência, então a chamada usa uma declaração suportada.

```vue annotate="add:2,5"
<script setup lang="ts">
function current(): number {
  return 1;
}
const currentValue = current();
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-deprecated": "warn"
    },
    "typeAware": true
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_deprecated.rs#L5)
