---
title: "type/no-base-to-string"
description: "Proibir a conversão em string de valores que recaem em Object.prototype.toString()."
---

# `type/no-base-to-string`

Proibir a conversão em string de valores que recaem em Object.prototype.toString().

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

plain é um objeto simples, então toString() recai em Object.prototype.toString().

```vue annotate="remove:2,3"
<script setup lang="ts">
const plain = { a: 1 };
plain.toString();
</script>
```

## Correto

label já é uma string, então toString() usa a conversão de string.

```vue annotate="add:2,3"
<script setup lang="ts">
const label = "a";
label.toString();
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-base-to-string": "warn"
    },
    "typeAware": true
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_base_to_string.rs#L5)
