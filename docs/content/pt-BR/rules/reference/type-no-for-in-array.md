---
title: "type/no-for-in-array"
description: "Proibir for-in sobre valores semelhantes a array."
---

# `type/no-for-in-array`

Proibir for-in sobre valores semelhantes a array.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

for-in percorre as chaves de um array em vez dos valores dos elementos.

```vue annotate="remove:4,5"
<script setup lang="ts">
const numbers = [1, 2, 3];
function loopsKeys(): void {
  for (const key in numbers) {
    console.log(key);
  }
}
</script>
```

## Correto

for-of percorre os elementos numéricos do array.

```vue annotate="add:4,5"
<script setup lang="ts">
const numbers = [1, 2, 3];
function loopsKeys(): void {
  for (const n of numbers) {
    console.log(n);
  }
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
      "type/no-for-in-array": "warn"
    },
    "typeAware": true
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_for_in_array.rs#L5)
