---
title: "type/use-unknown-in-catch-callback-variable"
description: "Exigir que a variável do callback de catch de uma Promise tenha uma anotação unknown explícita."
---

# `type/use-unknown-in-catch-callback-variable`

Exigir que a variável do callback de catch de uma Promise tenha uma anotação unknown explícita.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

O callback de catch nomeia error sem uma anotação unknown explícita.

```vue annotate="remove:5"
<script setup lang="ts">
function getNumber(): Promise<number> {
  return Promise.resolve(1);
}
getNumber().catch((error) => {
  console.log(error);
});
</script>
```

## Correto

O callback de catch anota error como unknown.

```vue annotate="add:5"
<script setup lang="ts">
function getNumber(): Promise<number> {
  return Promise.resolve(1);
}
getNumber().catch((error: unknown) => {
  console.log(error);
});
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/use-unknown-in-catch-callback-variable": "warn"
    },
    "typeAware": true
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/use_unknown_in_catch_callback_variable.rs#L5)
