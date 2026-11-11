---
title: "vue/max-template-depth"
description: "Limita a profundidade do template."
---

# `vue/max-template-depth`

Limita a profundidade do template.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O span está aninhado além do máximo configurado.

```vue annotate="remove:2,3,4,5,6,7,8"
<template>
  <div>
    <section>
      <div>
        <span>deep</span>
      </div>
    </section>
  </div>
</template>
```

## Correto

O template tem um único elemento.

```vue annotate="add:2"
<template>
  <p>ok</p>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/max-template-depth": "warn"
    },
    "ruleOptions": {
      "vue/max-template-depth": {
        "maxDepth": 3
      }
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/max_template_depth.rs#L13)
