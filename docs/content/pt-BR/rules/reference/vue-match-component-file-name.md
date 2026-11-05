---
title: "vue/match-component-file-name"
description: "Exige que o nome declarado do componente corresponda ao nome do arquivo."
---

# `vue/match-component-file-name`

Exige que o nome declarado do componente corresponda ao nome do arquivo.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

O nome do arquivo faz parte desta verificação. A comparação padrão converte o nome declarado do componente para PascalCase ou kebab-case e o compara com o nome do arquivo sem a extensão e sem outras transformações. Arquivos com várias definições de componentes não são verificados.

## Incorreto

O nome declarado `OtherPanel` não corresponde ao nome de `MyPanel.vue` sem a extensão, mesmo com as conversões de caixa aceitas.

`MyPanel.vue`

```vue annotate="remove:2"
<script setup lang="ts">
defineOptions({ name: "OtherPanel" });
</script>
```

## Correto

Nomeie o componente como `MyPanel` para que o nome declarado corresponda ao arquivo que o contém.

`MyPanel.vue`

```vue annotate="add:2"
<script setup lang="ts">
defineOptions({ name: "MyPanel" });
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
        "vue/match-component-file-name": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/match_component_file_name.rs#L23)
