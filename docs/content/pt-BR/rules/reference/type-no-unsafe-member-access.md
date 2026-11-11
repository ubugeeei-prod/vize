---
title: "type/no-unsafe-member-access"
description: "Proibir acesso a membros de valores tipados como any."
---

# `type/no-unsafe-member-access`

Proibir acesso a membros de valores tipados como any.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

anyValue é any, então ler member é um acesso a propriedade sem verificação.

```vue annotate="remove:2,3"
<script setup lang="ts">
declare const anyValue: any;
console.log(anyValue.member);
</script>
```

## Correto

box tem uma propriedade member conhecida, então o acesso é verificado.

```vue annotate="add:2,3"
<script setup lang="ts">
const box = { member: 1 };
console.log(box.member);
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unsafe-member-access": "warn"
    },
    "typeAware": true
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_member_access.rs#L5)
