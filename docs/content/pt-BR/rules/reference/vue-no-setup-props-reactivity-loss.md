---
title: "vue/no-setup-props-reactivity-loss"
description: "Proíbe cópias dos valores das props de setup que perdem as atualizações reativas."
---

# `vue/no-setup-props-reactivity-loss`

Proíbe cópias dos valores das props de setup que perdem as atualizações reativas.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

Verifica cópias de props no escopo raiz de `setup` da Options API e de `<script setup>`. A própria desestruturação reativa de props do Vue 3.5 é permitida; uma cópia posterior de uma prop desestruturada na raiz do script é reportada. Leituras dentro de outra função são permitidas.

## Incorreto

Copiar `props.count` na raiz do script mantém seu valor atual e não acompanha as atualizações posteriores da prop.

```vue annotate="remove:3"
<script setup lang="ts">
const props = defineProps({ count: Number });
const snapshot = props.count;
</script>
```

## Correto

Leia `props.count` em um getter computado para manter o resultado derivado ligado às mudanças da prop.

```vue annotate="add:2,4"
<script setup lang="ts">
import { computed } from "vue";
const props = defineProps({ count: Number });
const snapshot = computed(() => props.count);
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
        "vue/no-setup-props-reactivity-loss": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_setup_props_reactivity_loss.rs#L44)
