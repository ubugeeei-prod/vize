---
title: "script/no-lifecycle-after-await"
description: "Proibir hooks de ciclo de vida registrados após um await em setup()."
---

# `script/no-lifecycle-after-await`

Proibir hooks de ciclo de vida registrados após um await em setup().

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

`setup()` chama `onMounted()` depois de `await loadData()`; sem instância de componente atual nesse ponto, o hook nunca é registrado.

```vue annotate="remove:5"
<script lang="ts">
import { onMounted } from 'vue'
export default {
  async setup() {
    await loadData()
    onMounted(() => {})
  },
}
</script>
```

## Correto

`onMounted()` é registrado antes do primeiro `await`, enquanto `setup()` ainda roda de forma síncrona para o componente.

```vue annotate="add:6"
<script lang="ts">
import { onMounted } from 'vue'
export default {
  async setup() {
    onMounted(() => {})
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
      "script/no-lifecycle-after-await": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_lifecycle_after_await.rs#L45)
