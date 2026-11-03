---
title: "script/no-restricted-globals"
description: "Proibir referências a variáveis globais do ambiente de execução que devem passar por um encapsulamento tipado."
---

# `script/no-restricted-globals`

Proibir referências a variáveis globais do ambiente de execução que devem passar por um encapsulamento tipado.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | Nenhuma | Não | [Opções das regras](/rules/options.md) |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

O exemplo lê diretamente as variáveis globais restritas por padrão `process`, `localStorage` e `sessionStorage`, ignorando as funções auxiliares explícitas de configuração e armazenamento do projeto.

```vue annotate="remove:2,3,4"
<script setup lang="ts">
const flag = process.env.FEATURE_FLAG
const token = localStorage.getItem('auth.token')
sessionStorage.setItem('view.scroll', String(window.scrollY))
</script>
```

## Correto

`useFeatureFlag`, `authStorage.read` e `viewStorage.write` removem essas referências diretas às variáveis globais restritas. O `window.scrollY` restante não é uma restrição padrão desta regra; a segurança em SSR é uma questão separada.

```vue annotate="add:2,3,4,5,6,7"
<script setup lang="ts">
// Use a typed config helper that distinguishes server vs. client.
const flag = useFeatureFlag('FEATURE_FLAG')

// Use a typed wrapper that scopes keys and handles SSR / disabled storage.
const token = authStorage.read('auth.token')
viewStorage.write('view.scroll', String(window.scrollY))
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
        "script/no-restricted-globals": "error"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Opções das regras](/rules/options.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_restricted_globals.rs#L57)
