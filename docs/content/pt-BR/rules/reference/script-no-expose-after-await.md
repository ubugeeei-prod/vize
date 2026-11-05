---
title: "script/no-expose-after-await"
description: "Proibir expose() e defineExpose() após um await."
---

# `script/no-expose-after-await`

Proibir expose() e defineExpose() após um await.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

`defineExpose()` roda depois de um `await` de nível superior, quando não há instância de componente atual, então nada é exposto.

```vue annotate="remove:2"
<script setup lang="ts">
await loadData()
defineExpose({ reload })
</script>
```

## Correto

`defineExpose()` roda antes do primeiro `await`, enquanto a instância do componente ainda é a atual.

```vue annotate="add:3"
<script setup lang="ts">
defineExpose({ reload })
await loadData()
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
        "script/no-expose-after-await": "error"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_expose_after_await.rs#L34)
