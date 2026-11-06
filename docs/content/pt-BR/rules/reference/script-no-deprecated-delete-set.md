---
title: "script/no-deprecated-delete-set"
description: "Proibir $set e $delete do Vue 2, removidos no Vue 3."
---

# `script/no-deprecated-delete-set`

Proibir $set e $delete do Vue 2, removidos no Vue 3.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

`created()` chama o `this.$set` removido para adicionar `ready`.

```vue annotate="remove:4"
<script lang="ts">
export default {
  created() {
    this.$set(this.item, 'ready', true)
  }
}
</script>
```

## Correto

O mesmo campo é atribuído diretamente com `this.item.ready = true`.

```vue annotate="add:4"
<script lang="ts">
export default {
  created() {
    this.item.ready = true
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
      "script/no-deprecated-delete-set": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_delete_set.rs#L41)
