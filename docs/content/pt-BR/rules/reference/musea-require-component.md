---
title: "musea/require-component"
description: "Exigir um componente de destino resolvível no bloco &lt;art&gt;."
---

# `musea/require-component`

Exigir um componente de destino resolvível no bloco &lt;art&gt;.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Blocos art, variant e style de arquivos .art.vue do Musea.

## Incorreto

O bloco art fornece um título, mas não identifica o componente exibido na prévia.

```vue annotate="remove:1,2"
<art title="Button">
  <variant name="primary"><div /></variant>
</art>
```

## Correto

A variante renderiza o Button importado e identifica o destino do art.

```vue annotate="add:1,2,3,4,5,6"
<script setup>
import Button from "./Button.vue";
</script>

<art>
  <variant name="primary"><Button /></variant>
</art>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "musea/require-component": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Musea](../musea.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_component.rs#L11)
