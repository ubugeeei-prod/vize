---
title: "script/require-define-component"
description: "Exigir defineComponent() em exports padrão de SFCs TypeScript."
---

# `script/require-define-component`

Exigir defineComponent() em exports padrão de SFCs TypeScript.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

A SFC TypeScript exporta diretamente um objeto da Options API, portanto suas opções não recebem os tipos contextuais fornecidos por `defineComponent()`.

```vue annotate="remove:2,7"
<script lang="ts">
export default {
  props: { count: Number },
  mounted() {
    console.log(this.count)
  }
}
</script>
```

## Correto

Envolver o objeto com `defineComponent()` aplica a verificação TypeScript específica do Vue a props, emits, `this` e às demais opções.

```vue annotate="add:2,3,8"
<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  props: { count: Number },
  mounted() {
    console.log(this.count)
  }
})
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/require-define-component": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_define_component.rs#L48)
