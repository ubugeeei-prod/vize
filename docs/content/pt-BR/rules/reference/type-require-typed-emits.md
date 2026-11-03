---
title: "type/require-typed-emits"
description: "Exigir uma definição de tipo para defineEmits."
---

# `type/require-typed-emits`

Exigir uma definição de tipo para defineEmits.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

O `defineEmits(["save"])` que usa apenas um array declara o nome do evento sem um contrato tipado de carga útil.

```vue annotate="remove:2"
<script setup lang="ts">
defineEmits(["save"]);
</script>
```

## Correto

`defineEmits<{ save: [] }>()` declara o evento tipado `save` com uma tupla vazia de carga útil, indicando explicitamente que ele não recebe argumentos de carga útil.

```vue annotate="add:2"
<script setup lang="ts">
defineEmits<{ save: [] }>();
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "type/require-typed-emits": "warn"
      },
      "typeAware": true
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/require_typed_emits.rs#L54)
