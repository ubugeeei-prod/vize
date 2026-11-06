---
title: "script/no-with-defaults"
description: "Desencorajar withDefaults em favor de valores padrão na desestruturação (Vue 3.5+)."
---

# `script/no-with-defaults`

Desencorajar withDefaults em favor de valores padrão na desestruturação (Vue 3.5+).

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `opinionated` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

`withDefaults` envolve a declaração tipada de props apenas para fornecer valores padrão a `count` e `name`, em vez do estilo de valores padrão na desestruturação do Vue 3.5+ preferido aqui.

```vue annotate="remove:2"
<script setup lang="ts">
const props = withDefaults(defineProps<{ count?: number; name?: string }>(), { count: 0, name: "Ada" });
</script>
```

## Correto

O padrão de desestruturação coloca `count = 0` e `name = "Ada"` junto às suas variáveis e remove o encapsulamento de `withDefaults`.

```vue annotate="add:2"
<script setup lang="ts">
const { count = 0, name = "Ada" } = defineProps<{ count?: number; name?: string }>();
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-with-defaults": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_with_defaults.rs#L41)
