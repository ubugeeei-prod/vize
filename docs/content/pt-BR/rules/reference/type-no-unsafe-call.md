---
title: "type/no-unsafe-call"
description: "Proibir chamar valores tipados como any."
---

# `type/no-unsafe-call`

Proibir chamar valores tipados como any.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

anyValue tem tipo any, então chamá-lo não verifica o destino da chamada.

```vue annotate="remove:2,3"
<script setup lang="ts">
declare const anyValue: any;
anyValue();
</script>
```

## Correto

plainNumber é uma função, então a chamada tem um destino conhecido.

```vue annotate="add:2,3,4,5"
<script setup lang="ts">
function plainNumber(): number {
  return 1;
}
plainNumber();
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unsafe-call": "warn"
    },
    "typeAware": true
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_call.rs#L5)
