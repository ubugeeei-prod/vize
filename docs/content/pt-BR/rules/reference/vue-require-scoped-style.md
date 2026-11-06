---
title: "vue/require-scoped-style"
description: "Exigir o atributo scoped nas tags style."
---

# `vue/require-scoped-style`

Exigir o atributo scoped nas tags style.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

## Incorreto

O estilo `.button` não tem escopo e pode afetar elementos correspondentes fora deste componente.

```vue annotate="remove:1"
<style>
.button {
  color: red;
}
</style>
```

## Correto

Adicionar `scoped` aplica o escopo de componente do Vue ao mesmo seletor e às mesmas declarações.

```vue annotate="add:1"
<style scoped>
.button {
  color: red;
}
</style>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/require-scoped-style": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_scoped_style.rs#L49)
