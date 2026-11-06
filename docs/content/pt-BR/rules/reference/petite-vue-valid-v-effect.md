---
title: "petite-vue/valid-v-effect"
description: "Exigir uma expressão não vazia em v-effect."
---

# `petite-vue/valid-v-effect`

Exigir uma expressão não vazia em v-effect.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | Nenhuma | Não | Nenhuma |

Aplicável a: Documentos HTML detectados como petite-vue; SFCs Vue comuns ficam fora do escopo desta regra.

## Incorreto

Cada `v-effect` não tem uma expressão executável: seu valor está ausente, vazio ou contém apenas espaços em branco.

```html annotate="remove:3,4,5"
<!doctype html>
<html><body>
<div v-effect></div>
<div v-effect=""></div>
<div v-effect="   "></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

## Correto

Ambos os valores de `v-effect` contêm uma expressão: um atualiza `el.textContent` e o outro incrementa `count`. Esta regra verifica se há uma expressão não vazia, e não a lógica de negócio do efeito.

```html annotate="add:3,4"
<!doctype html>
<html><body>
<div v-effect="el.textContent = count"></div>
<div v-effect="count++"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "petite-vue/valid-v-effect": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de petite-vue](../petite-vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_effect.rs#L37)
