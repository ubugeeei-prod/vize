---
title: "script/return-in-emits-validator"
description: "Exigir um valor de retorno em todo validador de emits da Options API."
---

# `script/return-in-emits-validator`

Exigir um valor de retorno em todo validador de emits da Options API.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

Use uma função de seta com corpo em bloco para o filtro de SFC atualmente suportado. O validador subjacente também trata a sintaxe abreviada de métodos, mas o pré-filtro atual de SFC não encaminha essa forma de modo confiável.

## Incorreto

O validador `submit` registra a carga útil, mas não retorna um resultado de validação, então seu corpo de bloco produz undefined.

```vue annotate="remove:2"
<script lang="ts">
export default { emits: { submit: (payload: unknown) => { console.log(payload); } } };
</script>
```

## Correto

`return payload != null` fornece um resultado booleano de validação para a carga útil enviada em vez de encerrar sem um valor retornado.

```vue annotate="add:2"
<script lang="ts">
export default { emits: { submit: (payload: unknown) => { return payload != null; } } };
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/return-in-emits-validator": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/return_in_emits_validator.rs#L61)
