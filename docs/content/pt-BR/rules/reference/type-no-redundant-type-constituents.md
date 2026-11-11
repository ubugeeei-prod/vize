---
title: "type/no-redundant-type-constituents"
description: "Proibir membros de uniões e interseções que não acrescentam ou substituem informação de tipo."
---

# `type/no-redundant-type-constituents`

Proibir membros de uniões e interseções que não acrescentam ou substituem informação de tipo.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

unknown já cobre string, então o constituinte string não restringe a união.

```vue annotate="remove:2"
<script setup lang="ts">
type Redundant = unknown | string;
</script>
```

## Correto

string e number são ambos constituintes úteis da união.

```vue annotate="add:2"
<script setup lang="ts">
type Redundant = string | number;
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-redundant-type-constituents": "warn"
    },
    "typeAware": true
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_redundant_type_constituents.rs#L5)
