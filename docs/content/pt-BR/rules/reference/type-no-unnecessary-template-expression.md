---
title: "type/no-unnecessary-template-expression"
description: "Proibir template literals que envolvem uma única expressão sem necessidade."
---

# `type/no-unnecessary-template-expression`

Proibir template literals que envolvem uma única expressão sem necessidade.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

O template contém apenas label, então não constrói uma string nova.

```vue annotate="remove:3"
<script setup lang="ts">
const label = "a";
const wrapped = `${label}`;
</script>
```

## Correto

Atribuir label usa a string sem um invólucro de template.

```vue annotate="add:3"
<script setup lang="ts">
const label = "a";
const wrapped = label;
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unnecessary-template-expression": "warn"
    },
    "typeAware": true
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unnecessary_template_expression.rs#L5)
