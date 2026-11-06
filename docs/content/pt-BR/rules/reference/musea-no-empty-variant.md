---
title: "musea/no-empty-variant"
description: "Proibir blocos &lt;variant&gt; vazios."
---

# `musea/no-empty-variant`

Proibir blocos &lt;variant&gt; vazios.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Blocos art, variant e style de arquivos .art.vue do Musea.

## Incorreto

A variante chamada primary está vazia, então não fornece conteúdo para a prévia.

```vue annotate="remove:2"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

## Correto

A variante renderiza um Button primary com seu conteúdo Save.

```vue annotate="add:2,3,4"
<art title="Button" component="./Button.vue">
  <variant name="primary">
    <Button tone="primary">Save</Button>
  </variant>
</art>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "musea/no-empty-variant": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de Musea e CSS](../musea-and-css.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/no_empty_variant.rs#L8)
