---
title: "musea/require-title"
description: "Exigir o atributo title no bloco &lt;art&gt;."
---

# `musea/require-title`

Exigir o atributo title no bloco &lt;art&gt;.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | Nenhuma | Não | Nenhuma |

Aplicável a: Blocos art, variant e style de arquivos .art.vue do Musea.

## Incorreto

O bloco art identifica Button.vue, mas não fornece um título.

```vue annotate="remove:1"
<art component="./Button.vue">
  <variant name="primary" />
</art>
```

## Correto

As opções de defineArt fornecem o título Button para o bloco art.

```vue annotate="add:1,2,3,4,5"
<script setup>
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="primary" />
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
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_title.rs#L31)
