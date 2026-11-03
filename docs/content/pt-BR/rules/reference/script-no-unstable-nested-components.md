---
title: "script/no-unstable-nested-components"
description: "Proibir definições de componentes dentro de funções de setup ou renderização."
---

# `script/no-unstable-nested-components`

Proibir definições de componentes dentro de funções de setup ou renderização.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

`defineComponent` é executado dentro do `setup()` do pai, criando uma nova definição do componente `Child` sempre que esse setup é executado.

```vue annotate="remove:3"
<script lang="ts">
import { defineComponent } from "vue";
export default { setup() { const Child = defineComponent({ render() { return null; } }); return { Child }; } };
</script>
```

## Correto

A definição de `Child` é movida para o escopo do módulo, e `setup()` retorna essa definição existente em vez de recriá-la.

```vue annotate="add:3,4"
<script lang="ts">
import { defineComponent } from "vue";
const Child = defineComponent({ render() { return null; } });
export default { setup() { return { Child }; } };
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
        "script/no-unstable-nested-components": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_unstable_nested_components.rs#L19)
