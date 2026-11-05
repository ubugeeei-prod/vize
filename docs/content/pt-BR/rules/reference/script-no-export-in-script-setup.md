---
title: "script/no-export-in-script-setup"
description: "Proibir instruções export dentro de &lt;script setup&gt;."
---

# `script/no-export-in-script-setup`

Proibir instruções export dentro de &lt;script setup&gt;.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

`export const count` tenta expor uma exportação de módulo a partir de `<script setup>`, onde exportações em tempo de execução são proibidas.

```vue annotate="remove:2"
<script setup lang="ts">
export const count = 1;
</script>
```

## Correto

Remover `export` mantém `count` como uma variável de setup em vez de uma exportação de módulo.

```vue annotate="add:2"
<script setup lang="ts">
const count = 1;
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
        "script/no-export-in-script-setup": "error"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_export_in_script_setup.rs#L49)
