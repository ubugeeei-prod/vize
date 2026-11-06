---
title: "script/prefer-import-from-vue"
description: "Preferir importar de 'vue' em vez de pacotes internos."
---

# `script/prefer-import-from-vue`

Preferir importar de 'vue' em vez de pacotes internos.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Sim | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

`ref` e `h` são importados dos pacotes internos `@vue/runtime-core` e `@vue/runtime-dom` em vez do pacote público `vue`.

```vue annotate="remove:2,3"
<script setup lang="ts">
import { ref } from '@vue/runtime-core'
import { h } from '@vue/runtime-dom'
</script>
```

## Correto

As funções públicas são importadas de `vue`. A função privada `isString` mantém `@vue/shared`, pois `vue` não a exporta. Imports de namespace, padrão e mistos públicos/privados mantêm um aviso sem correção automática.

```vue annotate="add:2,3"
<script setup lang="ts">
import { ref, h } from 'vue'
import { isString } from '@vue/shared'
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/prefer-import-from-vue": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_from_vue.rs#L36)
