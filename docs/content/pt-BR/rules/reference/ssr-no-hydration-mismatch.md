---
title: "ssr/no-hydration-mismatch"
description: "Proibir valores não determinísticos que causam divergências na hidratação."
---

# `ssr/no-hydration-mismatch`

Proibir valores não determinísticos que causam divergências na hidratação.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O template avalia `Math.random()` durante a renderização, então o servidor e o cliente podem produzir textos diferentes para o mesmo parágrafo.

```vue annotate="remove:2"
<template>
  <p>{{ Math.random() }}</p>
</template>
```

## Correto

O parágrafo renderiza o estado estável `seed` em vez de um novo resultado aleatório. Neste exemplo no estilo do Nuxt, `useState` fornece o estado compartilhado e o inicializador é a constante `"stable"`.

```vue annotate="add:1,2,3,4,6"
<script setup lang="ts">
const seed = useState("seed", () => "stable");
</script>

<template>
  <p>{{ seed }}</p>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "ssr/no-hydration-mismatch": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de SSR](../ssr.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_hydration_mismatch.rs#L122)
