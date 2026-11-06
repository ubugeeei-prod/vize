---
title: "script/no-internal-imports"
description: "Proibir importações de módulos internos do Vue."
---

# `script/no-internal-imports`

Proibir importações de módulos internos do Vue.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

Ambas as importações apontam para arquivos internos de `dist` em vez do ponto de entrada público do pacote Vue, vinculando o componente aos caminhos dos arquivos de compilação.

```vue annotate="remove:2,3"
<script setup lang="ts">
import { foo } from '@vue/runtime-core/dist/runtime-core.esm-bundler'
import { bar } from 'vue/dist/vue.esm-bundler'
</script>
```

## Correto

Importar as funções auxiliares necessárias de `vue` remove a dependência dos locais dos arquivos internos de distribuição.

```vue annotate="add:2"
<script setup lang="ts">
import { ref, computed } from 'vue'
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-internal-imports": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_internal_imports.rs#L28)
