---
title: "script/prefer-define-options"
description: "Preferir defineOptions() a um &lt;script&gt; comum que só define name/inheritAttrs."
---

# `script/prefer-define-options`

Preferir defineOptions() a um &lt;script&gt; comum que só define name/inheritAttrs.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

A única instrução significativa do script comum exporta um objeto contendo apenas `name` e `inheritAttrs`; essas opções podem ser expressas por `defineOptions`.

```vue annotate="remove:2"
<script lang="ts">
export default { name: 'MyComponent', inheritAttrs: false }
</script>
```

## Correto

O método `data()` mostrado faz o script conter lógica efetiva da Options API, portanto ele fica fora da sugestão conservadora desta regra para scripts que contêm apenas opções. Este exemplo correto demonstra uma exceção permitida; a migração direta colocaria `defineOptions({ name: 'MyComponent', inheritAttrs: false })` em `<script setup>`.

```vue annotate="add:2,3,4,5,6"
<script lang="ts">
// Real options logic — keep the plain script.
export default {
  name: 'MyComponent',
  data() { return { count: 0 } },
}
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
        "script/prefer-define-options": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_define_options.rs#L52)
