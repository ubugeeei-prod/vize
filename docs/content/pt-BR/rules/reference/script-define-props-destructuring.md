---
title: "script/define-props-destructuring"
description: "Exigir um estilo consistente de desestruturação de defineProps em &lt;script setup&gt;."
---

# `script/define-props-destructuring`

Exigir um estilo consistente de desestruturação de defineProps em &lt;script setup&gt;.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Não | [Opções das regras](/rules/options.md) |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

A chamada tipada isolada a `defineProps` declara `foo` sem vinculá-lo segundo a política padrão de desestruturação.

```vue annotate="remove:2"
<script setup lang="ts">
defineProps<{ foo: string }>()
</script>
```

## Correto

O padrão de objeto vincula `foo` e `bar` diretamente e fornece um valor padrão para o `bar` opcional. Isso depende da desestruturação reativa de props do Vue 3.5+; o modo configurável `never` prefere a forma oposta.

```vue annotate="add:2"
<script setup lang="ts">
const { foo, bar = 'default' } = defineProps<{ foo: string; bar?: string }>()
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/define-props-destructuring": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Opções das regras](/rules/options.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_props_destructuring.rs#L30)
