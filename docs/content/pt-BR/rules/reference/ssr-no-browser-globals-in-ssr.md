---
title: "ssr/no-browser-globals-in-ssr"
description: "Proibir variáveis globais exclusivas do navegador no contexto de SSR."
---

# `ssr/no-browser-globals-in-ssr`

Proibir variáveis globais exclusivas do navegador no contexto de SSR.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

Setup lê `window.innerWidth` imediatamente, embora `window` não exista quando o componente é executado no servidor.

```vue annotate="remove:2"
<script setup lang="ts">
const width = window.innerWidth;
</script>
```

## Correto

A largura inicial é um valor de ref seguro no servidor, e o acesso ao navegador é movido para `onMounted`, que é executado no cliente em vez de durante o setup de SSR.

```vue annotate="add:2,3,4,5,6"
<script setup lang="ts">
const width = ref(0);

onMounted(() => {
  width.value = window.innerWidth;
});
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
        "ssr/no-browser-globals-in-ssr": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de SSR](../ssr.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_browser_globals_in_ssr.rs#L157)
