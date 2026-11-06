---
title: "nuxt/nuxt-config-keys-order"
description: "Preferir a ordem recomendada das propriedades de configuração do Nuxt."
---

# `nuxt/nuxt-config-keys-order`

Preferir a ordem recomendada das propriedades de configuração do Nuxt.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `nuxt` | Sim | Nenhuma |

Aplicável a: Arquivos de configuração do Nuxt (nuxt.config.ts).

## Incorreto

A configuração coloca `ssr` antes de `modules`, invertendo sua ordem na sequência de chaves de configuração do Nuxt recomendada pela regra.

`nuxt.config.ts`

```ts annotate="remove:1"
export default defineNuxtConfig({ ssr: true, modules: [] });
```

## Correto

Colocar `modules` antes de `ssr` preserva os dois valores e atende à ordem prescrita; a correção muda a disposição, sem alterar o significado de nenhuma das opções.

`nuxt.config.ts`

```ts annotate="add:1"
export default defineNuxtConfig({ modules: [], ssr: true });
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "nuxt/nuxt-config-keys-order": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras do ecossistema e Nuxt](../ecosystem.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/nuxt_config_keys_order.rs#L24)
