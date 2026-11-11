---
title: "type/no-generated-empty-object-type"
description: "Proibir tipos de objeto vazios gerados por operações de tipo."
---

# `type/no-generated-empty-object-type`

Proibir tipos de objeto vazios gerados por operações de tipo.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

Omit remove a única chave, então o alias resulta num tipo de objeto vazio.

```vue annotate="remove:2"
<script setup lang="ts">
type Emptied = Omit<{ a: number }, "a">;
</script>
```

## Correto

Omit remove a e ainda deixa b, então o alias não é um tipo de objeto vazio.

```vue annotate="add:2"
<script setup lang="ts">
type Kept = Omit<{ a: number; b: string }, "a">;
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-generated-empty-object-type": "warn"
    },
    "typeAware": true
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_generated_empty_object_type.rs#L5)
