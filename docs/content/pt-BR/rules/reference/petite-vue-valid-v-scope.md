---
title: "petite-vue/valid-v-scope"
description: "Exigir que v-scope vincule um objeto literal."
---

# `petite-vue/valid-v-scope`

Exigir que v-scope vincule um objeto literal.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | Nenhuma | Não | Nenhuma |

Aplicável a: Documentos HTML detectados como petite-vue; SFCs Vue comuns ficam fora do escopo desta regra.

## Incorreto

Os quatro valores não vazios de `v-scope` são um identificador, uma chamada, uma operação aritmética e um número; nenhum é analisado como um objeto literal.

```html annotate="remove:3,4,5,6"
<!doctype html>
<html><body>
<div v-scope="count"></div>
<div v-scope="foo()"></div>
<div v-scope="a + b"></div>
<div v-scope="123"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

## Correto

Um `v-scope` sem valor usa o escopo raiz. Os outros valores são objetos literais, incluindo o objeto entre parênteses, que a regra aceita.

```html annotate="add:3,4,5,6"
<!doctype html>
<html><body>
<div v-scope></div>
<div v-scope="{}"></div>
<div v-scope="{ count: 0 }"></div>
<div v-scope="({ count: 0 })"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "petite-vue/valid-v-scope": "error"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de petite-vue](../petite-vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_scope.rs#L46)
