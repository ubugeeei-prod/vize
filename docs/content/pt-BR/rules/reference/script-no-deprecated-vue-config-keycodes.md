---
title: "script/no-deprecated-vue-config-keycodes"
description: "Proibir Vue.config.keyCodes, removido no Vue 3."
---

# `script/no-deprecated-vue-config-keycodes`

Proibir Vue.config.keyCodes, removido no Vue 3.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

`Vue.config.keyCodes.enter` atribui um alias de código de tecla removido.

```vue annotate="remove:2"
<script setup lang="ts">
Vue.config.keyCodes.enter = 13
</script>
```

## Correto

O código de tecla é uma constante comum, `const enter = 13`.

```vue annotate="add:2"
<script setup lang="ts">
const enter = 13
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
        "script/no-deprecated-vue-config-keycodes": "error"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_vue_config_keycodes.rs#L22)
