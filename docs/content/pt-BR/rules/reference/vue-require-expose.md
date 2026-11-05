---
title: "vue/require-expose"
description: "Exige que componentes da Options API declarem explicitamente suas propriedades públicas com `expose`."
---

# `vue/require-expose`

Exige que componentes da Options API declarem explicitamente suas propriedades públicas com `expose`.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

Esta verificação trata de objetos de componentes da Options API. Uma lista `expose` explicitamente vazia é válida. Chamar `expose` em setup ou retornar uma função de renderização também satisfaz a verificação; mixins são isentos. A passagem pelo script não inspeciona um bloco `<script setup>` irmão, e `defineExpose` não é o alvo desta regra.

## Incorreto

O componente da Options API tem um método `focus` sem declarar explicitamente a interface de sua instância pública.

```vue annotate="remove:2"
<script lang="ts">
export default { methods: { focus() {} } };
</script>
```

## Correto

Uma lista `expose` disponibiliza explicitamente `focus` pela instância pública do componente.

```vue annotate="add:2"
<script lang="ts">
export default { expose: ["focus"], methods: { focus() {} } };
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
        "vue/require-expose": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_expose.rs#L24)
