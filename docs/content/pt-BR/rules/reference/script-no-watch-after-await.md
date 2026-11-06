---
title: "script/no-watch-after-await"
description: "Proibir watchers registrados após um await em setup() sem guardar a função de parada."
---

# `script/no-watch-after-await`

Proibir watchers registrados após um await em setup() sem guardar a função de parada.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

`watch()` é criado depois de `await loadData()` e a função de parada é descartada, então o watcher nunca é parado junto com o componente.

```vue annotate="remove:5"
<script lang="ts">
import { watch } from 'vue'
export default {
  async setup(props) {
    await loadData()
    watch(() => props.id, reload)
  },
}
</script>
```

## Correto

O watcher é criado antes do primeiro `await`, então o Vue o para quando o componente é desmontado.

```vue annotate="add:6"
<script lang="ts">
import { watch } from 'vue'
export default {
  async setup(props) {
    watch(() => props.id, reload)
    await loadData()
  },
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
      "script/no-watch-after-await": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_watch_after_await.rs#L44)
