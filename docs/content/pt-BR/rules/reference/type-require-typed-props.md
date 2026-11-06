---
title: "type/require-typed-props"
description: "Exigir uma definição de tipo para defineProps."
---

# `type/require-typed-props`

Exigir uma definição de tipo para defineProps.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

O `defineProps(["title"])` que usa apenas um array declara `title` pelo nome sem lhe atribuir um tipo.

```vue annotate="remove:2"
<script setup lang="ts">
defineProps(["title"]);
</script>
```

## Correto

`defineProps<{ title: string }>()` atribui a `title` um tipo string explícito em vez de uma declaração de tempo de execução que contém apenas o nome.

```vue annotate="add:2"
<script setup lang="ts">
defineProps<{ title: string }>();
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/require-typed-props": "warn"
    },
    "typeAware": true
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/require_typed_props.rs#L57)
