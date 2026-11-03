---
title: "nuxt/no-page-meta-runtime-values"
description: "Proibir valores do contexto de execução no nível de avaliação imediata de `definePageMeta`, que é extraído para um fragmento separado durante a compilação e executado antes do setup do componente."
---

# `nuxt/no-page-meta-runtime-values`

Proibir valores do contexto de execução no nível de avaliação imediata de `definePageMeta`, que é extraído para um fragmento separado durante a compilação e executado antes do setup do componente.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `nuxt` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

`useRoute()` é avaliado imediatamente durante a construção do objeto `definePageMeta`, embora a macro eleve esses metadados para fora do contexto de execução de setup.

```vue annotate="remove:2"
<script setup lang="ts">
definePageMeta({ title: useRoute() });
</script>
```

## Correto

`validate` recebe uma função de callback, então o acesso a `useRoute().params.id` é adiado até a execução dessa função. A regra distingue corpos de funções com execução adiada de valores de metadados avaliados imediatamente.

```vue annotate="add:2"
<script setup lang="ts">
definePageMeta({ validate: () => Boolean(useRoute().params.id) });
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
        "nuxt/no-page-meta-runtime-values": "error"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras do ecossistema e Nuxt](../ecosystem.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_page_meta_runtime_values.rs#L25)
