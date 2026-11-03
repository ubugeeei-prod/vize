---
title: "ecosystem/pinia-prefer-store-to-refs"
description: "Preferir storeToRefs() ao desestruturar stores do Pinia."
---

# `ecosystem/pinia-prefer-store-to-refs`

Preferir storeToRefs() ao desestruturar stores do Pinia.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `ecosystem` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

Desestruturar `name` diretamente do store separa o valor de seu acesso reativo ao store.

```vue annotate="remove:2"
<script setup lang="ts">
const { name } = useUserStore();
</script>
```

## Correto

O store permanece intacto e storeToRefs cria uma referência reativa para name.

```vue annotate="add:2,3"
<script setup lang="ts">
const store = useUserStore();
const { name } = storeToRefs(store);
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "ecosystem/pinia-prefer-store-to-refs": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras do ecossistema e Nuxt](../ecosystem.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/pinia_prefer_store_to_refs.rs#L20)
