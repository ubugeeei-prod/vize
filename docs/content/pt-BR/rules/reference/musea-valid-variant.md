---
title: "musea/valid-variant"
description: "Exigir o atributo name nos blocos &lt;variant&gt;."
---

# `musea/valid-variant`

Exigir o atributo name nos blocos &lt;variant&gt;.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | Nenhuma | Não | Nenhuma |

Aplicável a: Blocos art, variant e style de arquivos .art.vue do Musea.

## Incorreto

A variante omite o nome necessário para identificar a prévia.

```vue annotate="remove:2"
<art title="Button" component="./Button.vue">
  <variant />
</art>
```

## Correto

O nome primary identifica essa variante.

```vue annotate="add:2"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "musea/valid-variant": "error"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Musea](../musea.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/valid_variant.rs#L8)
