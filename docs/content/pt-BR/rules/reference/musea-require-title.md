---
title: "musea/require-title"
description: "Exigir um título de exibição resolvível no bloco &lt;art&gt;."
---

# `musea/require-title`

Exigir um título de exibição resolvível no bloco &lt;art&gt;.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | Nenhuma | Não | Nenhuma |

Aplicável a: Blocos art, variant e style de arquivos .art.vue do Musea.

## Incorreto

O bloco art não tem metadados explícitos nem um componente que permita inferir o título.

```vue annotate="remove:2"
<art>
  <variant name="primary"><div /></variant>
</art>
```

## Correto

O destino Button importado fornece o título de exibição inferido.

```vue annotate="add:1,2,3,4,6"
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
      "musea/require-title": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Musea](../musea.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_title.rs#L32)
