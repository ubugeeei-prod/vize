---
title: "script/no-potential-component-option-typo"
description: "Sinalizar prováveis erros de digitação nos nomes de opções de componentes da Options API."
---

# `script/no-potential-component-option-typo`

Sinalizar prováveis erros de digitação nos nomes de opções de componentes da Options API.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

A opção está escrita como `method`, a uma edição de distância da opção reconhecida `methods`; o Vue não a trataria como a declaração de métodos pretendida.

```vue annotate="remove:2"
<script lang="ts">
export default { method: { save() {} } };
</script>
```

## Correto

Alterar a chave para `methods` coloca `save()` dentro da opção de componente reconhecida.

```vue annotate="add:2"
<script lang="ts">
export default { methods: { save() {} } };
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
        "script/no-potential-component-option-typo": "error"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_potential_component_option_typo.rs#L19)
