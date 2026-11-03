---
title: "vue/max-template-complexity"
description: "Limitar a complexidade do próprio template de um componente, tanto ciclomática quanto cognitiva."
---

# `vue/max-template-complexity`

Limitar a complexidade do próprio template de um componente, tanto ciclomática quanto cognitiva.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

Bad tem complexidade ciclomática 13 e complexidade cognitiva 25 (limites: 11 e 16). Cada componente é medido separadamente; apenas templates HTML inline são suportados.

Veja [pontuação de complexidade e limites de componentes](../../guide/cross-file-complexity.md) para entender como as duas pontuações do exemplo são calculadas.

## Incorreto

As ramificações, o laço, o conteúdo de slot e as decisões em expressões escritos pelo componente pai produzem pontuações de 13 e 25, acima dos limites padrão de 11 e 16.

```vue annotate="remove:1,2,3,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19"
<script setup lang="ts">
defineProps<{ rows: Row[] }>();
</script>
<template>
  <section>
    <h1>{{ user ? user.name : 'Guest' }}</h1>
    <DataTable :rows="rows">
      <template #cell="{ row, column }">
        <span v-if="column.key === 'status'" :class="row.active ? 'on' : 'off'">{{ row.status ?? 'unknown' }}</span>
        <a v-else-if="column.key === 'link' && row.url" :href="row.url">{{ row.label }}</a>
        <template v-else>
          <em v-for="tag in row.tags" :key="tag">
            <b v-if="tag.pinned || tag.starred">{{ tag.hot ? '!' : '' }}</b>
          </em>
        </template>
      </template>
    </DataTable>
    <p v-if="!rows.length && !loading">No data</p>
  </section>
</template>
```

## Correto

O template pai delega a renderização a RowList e mantém um v-if; suas próprias pontuações são 2 e 1.

```vue annotate="add:2"
<template>
  <RowList v-if="ready" :rows="rows" />
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/max-template-complexity": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/facts/max_template_complexity.rs#L56)
