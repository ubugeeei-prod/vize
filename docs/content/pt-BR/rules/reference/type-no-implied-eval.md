---
title: "type/no-implied-eval"
description: "Proibir APIs de execução dinâmica baseadas em string."
---

# `type/no-implied-eval`

Proibir APIs de execução dinâmica baseadas em string.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

setTimeout recebe uma string, que é executada como código dinâmico.

```vue annotate="remove:2,3"
<script setup lang="ts">
function logVoid(): void {}
setTimeout("logVoid()", 10);
</script>
```

## Correto

setTimeout recebe uma função, então nenhuma string é avaliada.

```vue annotate="add:2"
<script setup lang="ts">
setTimeout(() => {}, 10);
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-implied-eval": "warn"
    },
    "typeAware": true
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_implied_eval.rs#L5)
