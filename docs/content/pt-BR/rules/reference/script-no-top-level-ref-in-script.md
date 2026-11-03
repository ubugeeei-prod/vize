---
title: "script/no-top-level-ref-in-script"
description: "Proibir ref/reactive no nível superior para evitar contaminação de estado entre requisições."
---

# `script/no-top-level-ref-in-script`

Proibir ref/reactive no nível superior para evitar contaminação de estado entre requisições.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | Nenhuma | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

O `<script>` comum inicializa `count` e `user` no escopo do módulo. Durante SSR, esses objetos de estado podem ser compartilhados entre instâncias de componentes e requisições.

```vue annotate="remove:1,2,4,8"
<script>
// This state is shared across all requests in SSR!
const count = ref(0)
const user = reactive({ name: '' })

export default {
  setup() {
    return { count, user }
  }
}
</script>
```

## Correto

O ref de setup é inicializado por instância de componente; o script comum mantém apenas uma constante, uma função que produz estado e um ref criado dentro de `setup()`. Nenhuma dessas formas cria estado reativo no escopo do módulo comum.

```vue annotate="add:1,2,4,6,7,8,9,10,11,12,13,14,17,18,19"
<script setup>
// Script setup creates fresh state per request
const count = ref(0)
</script>

<script>
// Constants are fine
const API_URL = 'https://api.example.com'

// Functions that create state are fine
function createState() {
  return reactive({ count: 0 })
}

export default {
  setup() {
    // Create state inside setup
    const count = ref(0)
    return { count }
  }
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
        "script/no-top-level-ref-in-script": "error"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_top_level_ref_in_script.rs#L63)
