---
title: "css/no-id-selectors"
description: "Desencorajar o uso de seletores de ID no CSS."
---

# `css/no-id-selectors`

Desencorajar o uso de seletores de ID no CSS.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Não | Nenhuma |

Aplicável a: Blocos style em CSS puro (blocos scss, sass, less e stylus são ignorados).

## Incorreto

`#submit` vincula a regra de estilo a um seletor de ID.

```vue annotate="remove:2"
<style scoped>
#submit {
  font-weight: 600;
}
</style>
```

## Correto

A classe `.submit` fornece um ponto de aplicação de estilo reutilizável sem um seletor de ID.

```vue annotate="add:2"
<style scoped>
.submit {
  font-weight: 600;
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
      "css/no-id-selectors": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras CSS](../css.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_id_selectors.rs#L19)
