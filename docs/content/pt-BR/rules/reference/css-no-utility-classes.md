---
title: "css/no-utility-classes"
description: "Alertar sobre a implementação de classes utilitárias nos estilos de componentes."
---

# `css/no-utility-classes`

Alertar sobre a implementação de classes utilitárias nos estilos de componentes.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Não | Nenhuma |

Aplicável a: Blocos style em CSS puro (blocos scss, sass, less e stylus são ignorados).

## Incorreto

Os seletores definidos usam nomes típicos de utilitários, como `.flex`, `.mt-4` e `.text-center`.

```vue annotate="remove:2,3,4"
<style scoped>
.flex { display: flex; }
.mt-4 { margin-top: 1rem; }
.text-center { text-align: center; }
</style>
```

## Correto

Um seletor `.my-component` específico do componente agrupa seus estilos sob um único nome semântico.

```vue annotate="add:2"
<style scoped>
.my-component { display: flex; margin-top: 1rem; }
</style>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "css/no-utility-classes": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de Musea e CSS](../musea-and-css.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_utility_classes.rs#L37)
