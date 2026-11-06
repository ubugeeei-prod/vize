---
title: "script/no-reserved-keys"
description: "Proibir nomes reservados pelo Vue como chaves de props/data/computed/methods/setup/inject na Options API."
---

# `script/no-reserved-keys`

Proibir nomes reservados pelo Vue como chaves de props/data/computed/methods/setup/inject na Options API.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

A chave de data retornada, `$el`, colide com a propriedade nativa da instância do componente Vue e também usa um prefixo `$` reservado.

```vue annotate="remove:2"
<script lang="ts">
export default { data() { return { $el: "custom" }; } };
</script>
```

## Correto

Renomear o dado da aplicação para `elementLabel` evita a API nativa da instância e o prefixo reservado.

```vue annotate="add:2"
<script lang="ts">
export default { data() { return { elementLabel: "custom" }; } };
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-reserved-keys": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reserved_keys.rs#L30)
