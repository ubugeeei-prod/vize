---
title: "type/no-unnecessary-boolean-literal-compare"
description: "Proibir comparações desnecessárias com literais booleanos."
---

# `type/no-unnecessary-boolean-literal-compare`

Proibir comparações desnecessárias com literais booleanos.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

flag já é boolean, então compará-lo com true não acrescenta uma verificação.

```vue annotate="remove:3"
<script setup lang="ts">
function isFlag(flag: boolean): boolean {
  return flag === true;
}
</script>
```

## Correto

Devolver flag usa o valor booleano diretamente.

```vue annotate="add:3"
<script setup lang="ts">
function isFlag(flag: boolean): boolean {
  return flag;
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
      "type/no-unnecessary-boolean-literal-compare": "warn"
    },
    "typeAware": true
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unnecessary_boolean_literal_compare.rs#L5)
