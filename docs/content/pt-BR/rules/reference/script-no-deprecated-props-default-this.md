---
title: "script/no-deprecated-props-default-this"
description: "Proibir `this` dentro de uma função de valor padrão ou validação de prop (removido no Vue 3)."
---

# `script/no-deprecated-props-default-this`

Proibir `this` dentro de uma função de valor padrão ou validação de prop (removido no Vue 3).

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

O valor padrão e o validador da prop leem `this`, mas essas funções não podem depender da instância do componente no Vue 3.

```vue annotate="remove:6,7,8,13,14"
<script lang="ts">
export default {
  props: {
    size: {
      type: Number,
      // `this` is not the component instance in Vue 3.
      default() {
        return this.defaultSize
      }
    },
    value: {
      type: Number,
      validator() {
        return this.value > 0
      }
    }
  }
}
</script>
```

## Correto

A função de valor padrão lê `props.baseSize` de seu argumento, e o validador testa seu argumento `value`. Ambos deixam de depender de um objeto receptor de instância indisponível.

```vue annotate="add:6,7,8,13,14"
<script lang="ts">
export default {
  props: {
    size: {
      type: Number,
      // Vue 3 passes the raw props as the first argument instead.
      default(props) {
        return props.baseSize
      }
    },
    value: {
      type: Number,
      validator(value) {
        return value > 0
      }
    }
  }
}
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-deprecated-props-default-this": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_props_default_this.rs#L71)
