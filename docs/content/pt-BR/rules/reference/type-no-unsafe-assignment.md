---
title: "type/no-unsafe-assignment"
description: "Proibir atribuir valores any a alvos mais específicos."
---

# `type/no-unsafe-assignment`

Proibir atribuir valores any a alvos mais específicos.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

anyValue é any e é atribuído a uma ligação anotada como number.

```vue annotate="remove:2,3"
<script setup lang="ts">
declare const anyValue: any;
const assigned: number = anyValue;
</script>
```

## Correto

A ligação é inicializada com o número 1.

```vue annotate="add:2"
<script setup lang="ts">
const assigned: number = 1;
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unsafe-assignment": "warn"
    },
    "typeAware": true
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_assignment.rs#L5)
